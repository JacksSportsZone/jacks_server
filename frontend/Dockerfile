# Build Svelte app
FROM node:18 AS builder
WORKDIR /app
COPY . .
RUN npm install && npm run build

# Serve with Nginx
FROM nginx:alpine
COPY --from=builder /app/build /usr/share/nginx/html
EXPOSE 80
