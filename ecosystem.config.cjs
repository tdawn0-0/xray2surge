module.exports = {
  apps: [{
    name: "xray2surge",
    script: "index.ts",
    interpreter: "bun",
    watch: true,
    ignore_watch: ["node_modules", "xray.json", ".git"],
    env: {
      NODE_ENV: "development",
    },
    env_production: {
      NODE_ENV: "production",
    }
  }]
};
