const { getDefaultConfig } = require('expo/metro-config');

const config = getDefaultConfig(__dirname);

// P0-3: Polyfill for @polkadot/api in React Native
config.resolver.extraNodeModules = {
  ...config.resolver.extraNodeModules,
  crypto: require.resolve('react-native-get-random-values'),
  stream: require.resolve('readable-stream'),
  buffer: require.resolve('buffer'),
};

// Enable package exports for proper ESM resolution
config.resolver.unstable_enablePackageExports = true;

// Transform @polkadot packages that use import.meta
// These packages need to be transformed by Metro instead of being treated as external
config.transformer = {
  ...config.transformer,
  getTransformOptions: async () => ({
    transform: {
      experimentalImportSupport: false,
      inlineRequires: true,
    },
  }),
};

// Ensure @polkadot packages are transformed
config.resolver.sourceExts = [...(config.resolver.sourceExts || []), 'cjs'];

module.exports = config;
