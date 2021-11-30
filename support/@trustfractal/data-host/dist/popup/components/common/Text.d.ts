import React from "react";
export declare enum TextSizes {
    SMALL = "var(--s-12)",
    MEDIUM = "var(--s-16)",
    LARGE = "var(--s-20)"
}
export declare enum TextHeights {
    SMALL = "var(--s-168)",
    MEDIUM = "var(--s-1875)",
    LARGE = "var(--s-23)",
    EXTRA_LARGE = "var(--s-26)"
}
export declare enum TextWeights {
    NORMAL = "normal",
    SEMIBOLD = "500",
    BOLD = "bold"
}
export declare type TextProps = {
    size: TextSizes;
    height: TextHeights;
    weight: TextWeights;
    span?: boolean;
    center?: boolean;
};
declare function Text(props: TextProps & React.HTMLAttributes<HTMLParagraphElement>): JSX.Element;
declare namespace Text {
    var defaultProps: {
        size: TextSizes;
        height: TextHeights;
        weight: TextWeights;
        span: boolean;
    };
}
export default Text;
