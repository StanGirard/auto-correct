FROM erikvl87/languagetool:latest

# Install curl and unzip for n-gram download (Alpine uses apk)
USER root
RUN apk add --no-cache curl unzip

# Copy n-gram download script
COPY download-ngrams.sh /download-ngrams.sh
RUN chmod +x /download-ngrams.sh

# Environment variables for configuration
ENV Java_Xms=512m
ENV Java_Xmx=1g
ENV langtool_pipelinePrewarming=true
ENV langtool_languageModel=/ngrams

# Port configuration
EXPOSE 8010

# Health check (longer start period for n-gram download)
HEALTHCHECK --interval=30s --timeout=10s --start-period=300s \
  CMD curl -f http://localhost:8010/v2/languages || exit 1

# Use our script as entrypoint
ENTRYPOINT ["/download-ngrams.sh"]
