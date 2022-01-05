/* tslint:disable */
/* eslint-disable */
/**
* @param {string} s
* @returns {string | undefined}
*/
export function build(s: string): string | undefined;
/**
* @param {string} mtree
* @param {string} s
* @returns {string | undefined}
*/
export function extend(mtree: string, s: string): string | undefined;
/**
* @param {string} mtree
* @param {any} leaves
* @returns {string | undefined}
*/
export function extend_multiple(mtree: string, leaves: any): string | undefined;
/**
* @param {string} mtree_a
* @param {string} mtree_b
* @returns {string | undefined}
*/
export function strict_extension_proof(mtree_a: string, mtree_b: string): string | undefined;
/**
* @param {string} tree
* @returns {string | undefined}
*/
export function prune_balanced(tree: string): string | undefined;
