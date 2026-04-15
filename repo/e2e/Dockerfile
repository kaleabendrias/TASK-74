FROM mcr.microsoft.com/playwright:v1.44.0-jammy

WORKDIR /app

# Copy package files first for layer caching
COPY package.json ./
RUN npm install --no-audit --no-fund

# Copy config and tests
COPY tsconfig.json playwright.config.ts ./
COPY tests/ ./tests/

# Default: run all tests
CMD ["npx", "playwright", "test", "--reporter=list"]
