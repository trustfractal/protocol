import React from "react";
interface Props {
    underline?: boolean;
    uppercase?: boolean;
    center?: boolean;
    fontSize?: string;
    lineHeight?: string;
}
declare function Subtitle(props: React.HTMLProps<HTMLHeadingElement> & Props): JSX.Element;
export declare function Subsubtitle(props: React.HTMLProps<HTMLHeadingElement> & Props): JSX.Element;
export default Subtitle;
