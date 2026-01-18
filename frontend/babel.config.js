module.exports = function (api) {
  api.cache(true);
  return {
    presets: ['babel-preset-expo'],
    plugins: [
      // Module resolver for path aliases
      [
        'module-resolver',
        {
          root: ['./'],
          alias: {
            '@': './src',
          },
        },
      ],
      // Transform import.meta for non-module scripts
      function transformImportMeta() {
        return {
          visitor: {
            MetaProperty(path) {
              // Replace import.meta with a fallback object
              if (
                path.node.meta.name === 'import' &&
                path.node.property.name === 'meta'
              ) {
                path.replaceWithSourceString(
                  '({ url: typeof document !== "undefined" && document.currentScript ? document.currentScript.src : "file://" })'
                );
              }
            },
          },
        };
      },
    ],
  };
};
