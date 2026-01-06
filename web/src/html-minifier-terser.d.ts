declare module "html-minifier-terser" {
  export interface Options {
    collapseBooleanAttributes?: boolean;
    collapseWhitespace?: boolean;
    conservativeCollapse?: boolean;
    decodeEntities?: boolean;
    html5?: boolean;
    ignoreCustomComments?: RegExp[];
    minifyCSS?: boolean;
    minifyJS?: boolean;
    removeAttributeQuotes?: boolean;
    removeComments?: boolean;
    removeOptionalTags?: boolean;
    removeRedundantAttributes?: boolean;
    removeScriptTypeAttributes?: boolean;
    removeStyleLinkTypeAttributes?: boolean;
    sortAttributes?: boolean;
    sortClassName?: boolean;
  }

  export function minify(html: string, options?: Options): string;
}
