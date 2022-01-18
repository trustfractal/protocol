import About from '@components/About';
import { environment } from '@popup/Environment';
import { RoutesPaths } from '@popup/routes';
import { getWindowsService } from '@services/Factory';
import { useNavigate } from 'react-router';

declare const DATA_HOST_VERSION: string;

function AboutScreen() {
  const navigate = useNavigate();
  let version = '1';
  try {
    version = DATA_HOST_VERSION;
  } catch (error) {
    console.log('Cannot get version of application.');
  }

  const onNext = () => navigate(RoutesPaths.PROTOCOL);
  const onClickFractalLink = () =>
    getWindowsService().createTab({
      url: environment.FRACTAL_WEBSITE_URL,
    });
  const onClickFractalTelegram = () =>
    getWindowsService().createTab({
      url: 'https://t.me/fractal_protocol',
    });

  return (
    <About
      onClickFractalLink={onClickFractalLink}
      onClickFractalTelegram={onClickFractalTelegram}
      version={version}
      onNext={onNext}
    />
  );
}

export default AboutScreen;
