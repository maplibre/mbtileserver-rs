import commonjs from '@rollup/plugin-commonjs';
import resolve from '@rollup/plugin-node-resolve';
import css from 'rollup-plugin-css-only';
import { terser } from 'rollup-plugin-terser';

export default {
	input: 'index.js',
	output: {
		file: 'dist/core.min.js'
	},
	plugins: [
        css({ output: 'core.min.css' }),
        resolve(),
        commonjs(),
		terser()
	]
};
