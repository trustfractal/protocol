import React from "react";

import styled, { css } from "styled-components";

export enum TextSizes {
  SMALL = "var(--s-12)",
  MEDIUM = "var(--s-16)",
  LARGE = "var(--s-20)",
}

export enum TextHeights {
  SMALL = "var(--s-168)",
  MEDIUM = "var(--s-1875)",
  LARGE = "var(--s-23)",
  EXTRA_LARGE = "var(--s-26)",
}

export enum TextWeights {
  NORMAL = "normal",
  SEMIBOLD = "500",
  BOLD = "bold",
}

const RootParagraph = styled.p<TextProps>`
  ${(props) =>
    css`
      font-size: ${props.size};
      line-height: ${props.height};
      font-weight: ${props.weight};
      text-align: ${props.center ? "center" : "inherit"};
    `}
`;

const RootSpan = styled.span<TextProps>`
  ${(props) =>
    css`
      font-size: ${props.size};
      line-height: ${props.height};
      font-weight: ${props.weight};
      text-align: ${props.center ? "center" : "inherit"};
    `}
`;

export type TextProps = {
  size: TextSizes;
  height: TextHeights;
  weight: TextWeights;
  span?: boolean;
  center?: boolean;
};

Text.defaultProps = {
  size: TextSizes.MEDIUM,
  height: TextHeights.MEDIUM,
  weight: TextWeights.NORMAL,
  span: false,
};

function Text(props: TextProps & React.HTMLAttributes<HTMLParagraphElement>) {
  const { children, span, ...otherProps } = props;

  if (span) {
    return <RootSpan {...otherProps}>{children}</RootSpan>;
  }

  return <RootParagraph {...otherProps}>{children}</RootParagraph>;
}

export default Text;
