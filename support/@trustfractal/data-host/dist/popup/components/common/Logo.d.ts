import React from "react";
export declare enum LogoSizes {
    SMALL = "small",
    MEDIUM = "medium"
}
declare type LogoProps = {
    clickable?: boolean;
    width?: string;
    height?: string;
    size: LogoSizes;
};
declare function Logo(props: LogoProps & React.HtmlHTMLAttributes<HTMLImageElement>): JSX.Element;
declare namespace Logo {
    var defaultProps: {
        clickable: boolean;
        size: LogoSizes;
    };
}
export default Logo;
