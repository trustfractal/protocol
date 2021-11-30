import React from "react";
export declare type ButtonProps = {
    loading: boolean;
    alternative: boolean;
    leftIcon?: JSX.Element;
    rightIcon?: JSX.Element;
};
export declare function Button(props: ButtonProps & React.ButtonHTMLAttributes<HTMLButtonElement>): JSX.Element;
export declare namespace Button {
    var defaultProps: {
        loading: boolean;
        alternative: boolean;
    };
}
export default Button;
