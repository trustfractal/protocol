const { pathsToModuleNameMapper } = require('ts-jest');
// In the following statement, replace `./tsconfig` with the path to your `tsconfig` file
// which contains the path mapping (ie the `compilerOptions.paths` option):
const { compilerOptions } = require('./tsconfig.test.json');

module.exports = {
  roots: [
    "<rootDir>/src"
  ],
  testRegex: "(\\.(test|spec))\\.(ts|tsx|js)$",
  moduleNameMapper: pathsToModuleNameMapper(compilerOptions.paths , { prefix: '<rootDir>/' }  ),
  preset: 'ts-jest/presets/js-with-babel',
  transformIgnorePatterns: ['<rootDir>/node_modules/(?!@polkadot|@babel/runtime/helpers/esm/)'],
  setupFilesAfterEnv: ["<rootDir>/setupTests.ts"],
  globals: {
    'ts-jest': {
      tsconfig: 'tsconfig.test.json'
    }
  }
};
