import { AlertsDisplay } from '@components/Alerts';
import { Router } from '@popup/routes';
import GlobalStyle from '@styles/GlobalStyle';

function App() {
  return (
    <>
      <AlertsDisplay />
      <GlobalStyle />
      <Router />
    </>
  );
}

export default App;
