import Protocol from '@components/Protocol';
import AboutScreen from "@containers/AboutScreen";
import {
  BrowserRouter,
  Route,
  Routes
} from "react-router-dom";

import RoutesPaths from "@popup/routes/paths";

const Router = () => (
  <BrowserRouter>
    <Routes>
      <Route path={RoutesPaths.PROTOCOL} element={<Protocol/>} />
      <Route path={RoutesPaths.POPUP} element={<Protocol/>} />
      <Route path={RoutesPaths.ABOUT} element={<AboutScreen />} />
    </Routes>
  </BrowserRouter>
);

export default Router;
