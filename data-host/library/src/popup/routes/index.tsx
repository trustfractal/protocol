import Protocol from '@components/Protocol';
import { SendTokens } from '@components/SendTokens';
import { SweepTokens } from '@components/SweepTokens';
import AboutScreen from '@containers/AboutScreen';
import { BrowserRouter, Route, Routes } from 'react-router-dom';

export const Router = () => {
  return (
    <BrowserRouter>
      <Routes>
        <Route path={RoutesPaths.PROTOCOL} element={<Protocol />} />
        <Route path={RoutesPaths.POPUP} element={<Protocol />} />
        <Route path={RoutesPaths.ABOUT} element={<AboutScreen />} />
        <Route path={RoutesPaths.SWEEP} element={<SweepTokens />} />
        <Route path={RoutesPaths.SEND} element={<SendTokens />} />
      </Routes>
    </BrowserRouter>
  );
};

export enum RoutesPaths {
  PROTOCOL = '/',
  ABOUT = '/about',
  POPUP = '/popup.html',
  SWEEP = '/sweep_funds',
  SEND = '/send',
}
