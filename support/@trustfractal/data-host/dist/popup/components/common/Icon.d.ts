import React from "react";
export declare enum IconNames {
    LOGO = "logo",
    LOGO_SMALL = "logo-small",
    LOGO_NAME = "logo-name",
    FRACTAL_FULL_LOGO = "fractal-full-logo",
    WELCOME = "welcome",
    PROTOCOL = "protocol",
    PROTOCOL_SETUP_SUCCESS = "protocol-setup-success",
    PROTOCOL_SETUP_FAILURE = "protocol-setup-failure",
    PROTOCOL_SETUP_PENDING = "protocol-setup-pending"
}
declare type IconProps = {
    name: string;
    clickable: boolean;
    width?: string;
    height?: string;
};
declare function Icon(props: IconProps & React.HtmlHTMLAttributes<HTMLImageElement>): JSX.Element;
declare namespace Icon {
    var defaultProps: {
        clickable: boolean;
    };
}
export default Icon;
