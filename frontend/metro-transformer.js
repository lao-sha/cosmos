let upstream;

try {
  upstream = require('@expo/metro-config/build/babel-transformer');
} catch (e) {
  try {
    upstream = require('@expo/metro/metro-babel-transformer');
  } catch (e2) {
    upstream = require('metro-babel-transformer');
  }
}

function patchImportMeta(src, filename) {
  // Metro outputs a non-module bundle on web; `import.meta` will crash parsing.
  // Patch ALL files that contain import.meta, not just Polkadot
  if (!src.includes('import.meta')) return src;

  // Replace all import.meta usage
  return src
    .replace(/import\.meta\.url/g, '"about:blank"')
    .replace(/import\.meta/g, '({})');
}

module.exports.transform = function transform(params) {
  const patched = patchImportMeta(params.src, params.filename);
  const transformImpl = typeof upstream === 'function' ? upstream : upstream.transform;
  return transformImpl({ ...params, src: patched });
};
