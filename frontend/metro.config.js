const { getDefaultConfig } = require("expo/metro-config");

/** @type {import('expo/metro-config').MetroConfig} */
const config = getDefaultConfig(__dirname);

// Add support for .mjs files (often used by @polkadot/api)
config.resolver.sourceExts.push("mjs");

// Support package exports resolution
config.resolver.unstable_enablePackageExports = true;

// Patch import.meta usage in @polkadot/* packages for web bundling
config.transformer.babelTransformerPath = require.resolve("./metro-transformer");

module.exports = config;
