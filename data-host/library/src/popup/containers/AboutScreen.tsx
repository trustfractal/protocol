
import About from "@components/About";
import { environment } from "@popup/Environment";
import RoutesPaths from "@popup/routes/paths";
import { getWindowsService } from "@services/Factory";
import { useNavigate } from "react-router";

function AboutScreen() {
  const navigate = useNavigate();
  //TODO(melatron): Do we need a version?
  const version = 1;

  const onNext = () => navigate(RoutesPaths.PROTOCOL);
  const onClickFractalLink = () =>
    getWindowsService().createTab({
      url: environment.FRACTAL_WEBSITE_URL,
    });
  const onClickFractalTelegram = () =>
    getWindowsService().createTab({
      url: "https://t.me/fractal_protocol",
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
