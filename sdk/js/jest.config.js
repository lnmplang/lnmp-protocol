module.exports = {
    preset: 'ts-jest',
    testEnvironment: 'node',
    roots: ['<rootDir>/packages'],
    testMatch: ['**/__tests__/**/*.test.ts', '**/?(*.)+(spec|test).ts'],
    collectCoverageFrom: [
        'packages/*/src/**/*.ts',
        '!packages/*/src/**/*.d.ts',
        '!packages/*/src/**/index.ts',
    ],
    coverageThreshold: {
        global: {
            branches: 90,
            functions: 90,
            lines: 90,
            statements: 90,
        },
    },
    moduleNameMapper: {
        '^@lnmplang/core$': '<rootDir>/packages/core/src',
        '^@lnmplang/codec$': '<rootDir>/packages/codec/src',
        '^@lnmplang/lnmp$': '<rootDir>/packages/lnmp/src',
    },
};
