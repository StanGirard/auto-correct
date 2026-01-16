/**
 * Position mapping utility for contenteditable elements
 *
 * Problem: innerText adds \n for <br> and block elements, but DOM text nodes don't contain these.
 * LanguageTool returns offsets based on innerText, so we need to map these to actual DOM positions.
 *
 * Example:
 *   <p>Ligne un</p><p>Cest une erreur</p>
 *   innerText = "Ligne un\nCest une erreur" (offset of "Cest" = 9)
 *   DOM text nodes = ["Ligne un", "Cest une erreur"] (actual offset = 0 in second node)
 */

export interface DOMPosition {
  node: Text
  offset: number
}

/**
 * Builds a map from innerText offsets to DOM text node positions
 * Accounts for virtual \n characters from <br> and block elements
 */
export function buildPositionMap(element: HTMLElement): Map<number, DOMPosition> {
  const map = new Map<number, DOMPosition>()
  let innerTextOffset = 0
  let isFirstBlock = true

  function isBlockElement(node: Node): boolean {
    if (node.nodeType !== Node.ELEMENT_NODE) return false
    const el = node as Element

    // <br> acts as a newline
    if (el.tagName === 'BR') return true

    // Check computed display style for block elements
    const display = window.getComputedStyle(el).display
    return display === 'block' || display === 'list-item' || display === 'table' ||
           display === 'table-row' || display === 'flex' || display === 'grid'
  }

  function walk(node: Node): void {
    if (node.nodeType === Node.TEXT_NODE) {
      const text = node as Text
      const content = text.textContent || ''

      // Map each character position
      for (let i = 0; i < content.length; i++) {
        map.set(innerTextOffset++, { node: text, offset: i })
      }
    } else if (node.nodeType === Node.ELEMENT_NODE) {
      const el = node as Element

      // Handle <br> - adds a virtual \n but no actual text content
      if (el.tagName === 'BR') {
        innerTextOffset++ // Account for virtual \n
        return
      }

      const isBlock = isBlockElement(node)

      // Block elements add a virtual \n BEFORE their content (except first block)
      if (isBlock && !isFirstBlock) {
        innerTextOffset++ // Account for virtual \n between blocks
      }

      if (isBlock) {
        isFirstBlock = false
      }

      // Walk children
      el.childNodes.forEach(child => walk(child))
    }
  }

  walk(element)
  return map
}

/**
 * Gets the DOM position for a given innerText offset
 */
export function getPositionFromMap(
  map: Map<number, DOMPosition>,
  offset: number
): DOMPosition | null {
  return map.get(offset) || null
}

/**
 * Finds the DOM range for a given innerText offset and length
 * Returns both start and end positions
 */
export function getRangeFromMap(
  map: Map<number, DOMPosition>,
  offset: number,
  length: number
): { start: DOMPosition; end: DOMPosition } | null {
  const startPos = getPositionFromMap(map, offset)
  if (!startPos) return null

  // For end position, we want the position of the last character
  const endPos = getPositionFromMap(map, offset + length - 1)
  if (!endPos) return null

  return { start: startPos, end: endPos }
}
