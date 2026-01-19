FROM node:20-alpine

WORKDIR /app

# Copy and build
COPY main.ts ./

# Install TypeScript and build
RUN npm init -y && \
    npm install typescript @types/node && \
    npx tsc main.ts --target ES2022 --module NodeNext --moduleResolution NodeNext --outDir dist

ENV PORT=8080
EXPOSE 8080

CMD ["node", "dist/main.js"]
