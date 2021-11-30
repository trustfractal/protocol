import React from "react";
declare type TopComponentProps = {
    paddingTop: string;
    paddingBottom: string;
    paddingLeft: string;
    paddingRight: string;
};
declare function TopComponent(props: TopComponentProps & React.HTMLProps<HTMLDivElement>): JSX.Element;
declare namespace TopComponent {
    var defaultProps: {
        paddingTop: string;
        paddingBottom: string;
        paddingLeft: string;
        paddingRight: string;
    };
}
export default TopComponent;
