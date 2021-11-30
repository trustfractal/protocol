import React from "react";
export declare type InputProps = {
    underlined?: boolean;
    onEnter?: () => void;
    error?: string;
    label?: string;
    hint?: string;
};
declare function Input(props: InputProps & React.InputHTMLAttributes<HTMLInputElement>): JSX.Element;
declare namespace Input {
    var defaultProps: {
        underlined: boolean;
        onEnter: () => void;
    };
}
export default Input;
