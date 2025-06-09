# Frontend Dockerfile for React app
FROM node:18-alpine as builder

WORKDIR /app

# Copy package files
COPY package*.json ./

# Install dependencies
RUN npm ci

# Copy source code
COPY src/web ./src/web
COPY src/api/wordlist ./src/api/wordlist
COPY scripts ./scripts
COPY tsconfig.json tsconfig.node.json vite.config.js ./

# Generate wordlist and build the app  
RUN npm run generate-wordlist && npm run build

# Production stage with nginx
FROM nginx:alpine

# Copy built app to nginx
COPY --from=builder /app/src/web/dist /usr/share/nginx/html

# Copy nginx configuration
COPY src/web/nginx.conf /etc/nginx/conf.d/default.conf

# Expose port 80
EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]