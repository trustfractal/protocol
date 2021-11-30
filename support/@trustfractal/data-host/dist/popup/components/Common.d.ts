import { ButtonProps } from "@common/Button";
export { default as Text } from "@common/Text";
export { default as Subtitle, Subsubtitle, } from "@common/Subtitle";
export { default as Icon, IconNames } from "@common/Icon";
export { default as Input } from "@common/Input";
export { default as Title } from "@common/Title";
export declare const ClickableText: import("styled-components").StyledComponent<"button", any, {}, never>;
export declare function VerticalSequence(props: React.HTMLProps<HTMLDivElement>): JSX.Element;
export declare function Cta(props: React.ButtonHTMLAttributes<HTMLButtonElement> & ButtonProps): JSX.Element;
export declare namespace Cta {
    var defaultProps: {
        loading: boolean;
        alternative: boolean;
    };
}
export declare function BoldText({ children, center, }: {
    children: React.ReactNode;
    center?: boolean;
}): JSX.Element;
