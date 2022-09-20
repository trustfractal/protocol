import React from "https://unpkg.com/es-react@latest/dev/react.js";
import ReactDOM from "https://unpkg.com/es-react@latest/dev/react-dom.js";
import PropTypes from "https://unpkg.com/es-react@latest/dev/prop-types.js";
import htm from "https://unpkg.com/htm@latest?module";
import { ethers } from "https://cdnjs.cloudflare.com/ajax/libs/ethers/5.7.1/ethers.esm.js";

const html = htm.bind(React.createElement);

export {
  React,
  ReactDOM,
  PropTypes,
  html,
  ethers,
};
