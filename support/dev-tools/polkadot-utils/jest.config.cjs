const { defaults } = require('jest-config');

module.exports = {
  modulePaths: [
    "<rootDir>/src"
  ],
  moduleFileExtensions: [
    ...defaults.moduleFileExtensions,
    'ts',
    'tsx'
  ],
  testMatch: [
    '**/__tests__/**/*.[jt]s?(x)',
    '**/?(*.)+(spec|test).[jt]s?(x)'
  ],
  transform: {
    '^.+\\.(js|jsx|ts|tsx)$': require.resolve('babel-jest')
  },
  transformIgnorePatterns: [
    '/node_modules/(?!@polkadot|@babel/runtime/helpers/esm/)'
  ],
  verbose: true,
};
