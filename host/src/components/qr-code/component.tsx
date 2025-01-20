import QRCode from "react-qr-code";

function QRCodeBanner() {
  return (
    <div className="absolute bottom-4 left-4">
      <div className="bg-white p-3 rounded-lg shadow-xl flex flex-col items-center">
        <QRCode value="sadklfjasdklf" size={64} />
      </div>
    </div>
  );
}

export default QRCodeBanner;
