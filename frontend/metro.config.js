const { getDefaultConfig } = require('expo/metro-config');

const config = getDefaultConfig(__dirname);

// P0-3: Polyfill for @polkadot/api in React Native
config.resolver.extraNodeModules = {
  ...config.resolver.extraNodeModules,
  crypto: require.resolve('react-native-get-random-values'),
  stream: require.resolve('readable-stream'),
  buffer: require.resolve('buffer'),
};

// Fix for web: unstable_enablePackageExports
config.resolver.unstable_enablePackageExports = true;

module.exports = config;
