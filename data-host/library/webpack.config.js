const path = require('path');
const TsconfigPathsPlugin = require('tsconfig-paths-webpack-plugin');
const { CleanWebpackPlugin } = require('clean-webpack-plugin');
module.exports = {

    target: 'web',
  entry: './src/index.ts',
//   devtool: 'inline-source-map',
  plugins: [
    new CleanWebpackPlugin({
      cleanStaleWebpackAssets: false,
      cleanOnceBeforeBuildPatterns: [path.resolve(__dirname, './dist')],
    }),
  ],
  module: {
    rules: [
      {
        test: /\.tsx?$/,
        use: 'ts-loader',
        exclude: /node_modules/,
      },
      {
        test: /\.svg$/,
        use: [
          {
            loader: "@svgr/webpack",
            options: {
              svgo: false,
            },
          },
        ],
      },
      {
        test: /\.wasm$/,
        type: "webassembly/async"
      }
    ],
  },
  resolve: {
    extensions: ['.tsx', '.ts', '.js'],
    plugins: [new TsconfigPathsPlugin({/* options: see below */})],
    fallback: {
        "fs": false,
        "tls": false,
        "net": false,
        "path": false,
        "zlib": false,
        "http": false,
        "https": false,
        "stream": false,
        "crypto": false,
        "crypto-browserify": require.resolve('crypto-browserify'), //if you want to use this module also don't forget npm i crypto-browserify
      }
  },
  output: {
    filename: 'index.js',
    path: path.resolve(__dirname, 'dist'),
    library: 'DataHost',
    libraryTarget: 'umd',
    globalObject: 'this',
    umdNamedDefine: true,
  },
  experiments: {
    asyncWebAssembly: true,
    syncWebAssembly: true,
    // buildHttp: true,
    // layers: true,
    // lazyCompilation: true,
    // outputModule: true,
    // futureDefaults: true,
    topLevelAwait: true,
  },
};
