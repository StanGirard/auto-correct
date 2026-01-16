/**
 * Position mapping utility for contenteditable elements
 * Uses TreeWalker like LanguageTool for reliable text node traversal
 */

export interface DOMPosition {
  node: Text
  offset: number
}

export interface TextNodeInfo {
  node: Text
  start: number  // Start offset in plaintext
  end: number    // End offset in plaintext
}

/**
 * Get all text nodes in document order using TreeWalker
 * This matches how innerText traverses the DOM
 */
function getTextNodes(element: HTMLElement): Text[] {
  const textNodes: Text[] = []
  const walker = document.createTreeWalker(
    element,
    NodeFilter.SHOW_TEXT,
    {
      acceptNode: (node) => {
        // Skip text nodes inside script, style, etc.
        const parent = node.parentElement
        if (!parent) return NodeFilter.FILTER_REJECT

        const tagName = parent.tagName.toUpperCase()
        if (tagName === 'SCRIPT' || tagName === 'STYLE' ||
            tagName === 'NOSCRIPT' || tagName === 'TEMPLATE') {
          return NodeFilter.FILTER_REJECT
        }

        // Skip hidden elements
        const style = window.getComputedStyle(parent)
        if (style.display === 'none' || style.visibility === 'hidden') {
          return NodeFilter.FILTER_REJECT
        }

        return NodeFilter.FILTER_ACCEPT
      }
    }
  )

  let node: Text | null
  while ((node = walker.nextNode() as Text | null)) {
    textNodes.push(node)
  }

  return textNodes
}

/**
 * Find the text node and offset for a given plaintext offset
 * Uses the actual innerText to ensure correct mapping
 */
export function findPositionInDOM(
  element: HTMLElement,
  targetOffset: number
): DOMPosition | null {
  const innerText = element.innerText || ''

  // Validate offset
  if (targetOffset < 0 || targetOffset >= innerText.length) {
    return null
  }

  const textNodes = getTextNodes(element)
  let currentOffset = 0

  for (const textNode of textNodes) {
    const nodeText = textNode.textContent || ''
    const nodeStart = currentOffset
    const nodeEnd = currentOffset + nodeText.length

    // Check if target offset falls within this text node
    if (targetOffset >= nodeStart && targetOffset < nodeEnd) {
      return {
        node: textNode,
        offset: targetOffset - nodeStart
      }
    }

    currentOffset = nodeEnd

    // Account for newlines between block elements
    // innerText adds \n after block elements
    const parent = textNode.parentElement
    if (parent) {
      const nextSibling = getNextVisibleSibling(textNode)
      if (nextSibling && isBlockBoundary(textNode, nextSibling)) {
        // There's an implicit newline here
        if (targetOffset === currentOffset) {
          // Target is at the newline position - return end of current node
          return {
            node: textNode,
            offset: nodeText.length
          }
        }
        currentOffset++ // Account for the newline
      }
    }
  }

  // If we reach here, offset might be at the very end
  if (textNodes.length > 0 && targetOffset === currentOffset) {
    const lastNode = textNodes[textNodes.length - 1]
    return {
      node: lastNode,
      offset: lastNode.textContent?.length || 0
    }
  }

  return null
}

/**
 * Get the next visible sibling (skipping empty text nodes)
 */
function getNextVisibleSibling(node: Node): Node | null {
  let sibling = node.nextSibling
  while (sibling) {
    if (sibling.nodeType === Node.TEXT_NODE) {
      if ((sibling.textContent || '').trim()) {
        return sibling
      }
    } else if (sibling.nodeType === Node.ELEMENT_NODE) {
      return sibling
    }
    sibling = sibling.nextSibling
  }

  // Check parent's next sibling
  const parent = node.parentElement
  if (parent && parent.nextElementSibling) {
    return parent.nextElementSibling
  }

  return null
}

/**
 * Check if there's a block boundary between two nodes
 */
function isBlockBoundary(node1: Node, node2: Node): boolean {
  const parent1 = node1.parentElement
  const parent2 = node2.nodeType === Node.ELEMENT_NODE
    ? node2 as Element
    : (node2 as Text).parentElement

  if (!parent1 || !parent2) return false

  // Check if either parent is a block element
  const isBlock = (el: Element) => {
    const display = window.getComputedStyle(el).display
    return display === 'block' || display === 'list-item' ||
           display === 'flex' || display === 'grid' ||
           el.tagName === 'BR' || el.tagName === 'P' ||
           el.tagName === 'DIV' || el.tagName === 'LI'
  }

  // Different block parents = boundary
  if (parent1 !== parent2) {
    if (isBlock(parent1) || isBlock(parent2)) {
      return true
    }
  }

  return false
}

/**
 * Find DOM range for a given plaintext offset and length
 */
export function findRangeInDOM(
  element: HTMLElement,
  offset: number,
  length: number
): Range | null {
  const startPos = findPositionInDOM(element, offset)
  if (!startPos) {
    console.warn('[AutoCorrect] Could not find start position for offset:', offset)
    return null
  }

  const endPos = findPositionInDOM(element, offset + length)
  if (!endPos) {
    // Try end of selection (last character position)
    const endPosAlt = findPositionInDOM(element, offset + length - 1)
    if (!endPosAlt) {
      console.warn('[AutoCorrect] Could not find end position for offset:', offset + length)
      return null
    }
    // Adjust to be after the last character
    const range = document.createRange()
    range.setStart(startPos.node, startPos.offset)
    range.setEnd(endPosAlt.node, Math.min(endPosAlt.offset + 1, endPosAlt.node.length))
    return range
  }

  const range = document.createRange()
  range.setStart(startPos.node, startPos.offset)
  range.setEnd(endPos.node, endPos.offset)
  return range
}

// Legacy exports for compatibility
export function buildPositionMap(element: HTMLElement): Map<number, DOMPosition> {
  const map = new Map<number, DOMPosition>()
  const innerText = element.innerText || ''

  for (let i = 0; i < innerText.length; i++) {
    const pos = findPositionInDOM(element, i)
    if (pos) {
      map.set(i, pos)
    }
  }

  return map
}

export function getPositionFromMap(
  map: Map<number, DOMPosition>,
  offset: number
): DOMPosition | null {
  return map.get(offset) || null
}
