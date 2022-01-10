import Protocol from '@components/Protocol';
import { SendTokens } from "@components/SendTokens";
import { SweepTokens } from "@components/SweepTokens";
import AboutScreen from "@containers/AboutScreen";
import RoutesPaths from "@popup/routes/paths";
import {
  BrowserRouter,
  Route,
  Routes,
} from "react-router-dom";


const Router = () => {
  return (
    <BrowserRouter>
        <Routes>
        <Route path={RoutesPaths.PROTOCOL} element={<Protocol/>} />
        <Route path={RoutesPaths.POPUP} element={<Protocol/>} />
        <Route path={RoutesPaths.ABOUT} element={<AboutScreen />} />
        <Route path={RoutesPaths.SWEEP} element={<SweepTokens onFinish={RoutesPaths.PROTOCOL} />} />
        <Route path={RoutesPaths.SEND} element={<SendTokens onFinish={RoutesPaths.PROTOCOL} />} />
        </Routes>
    </BrowserRouter>
  );
};

export default Router;
