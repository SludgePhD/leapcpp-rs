#include <Leap.h>

namespace Leap {

struct RustListenerCallbacks {
    void (*onInit)(void* userdata, const Controller&);
    void (*onConnect)(void* userdata, const Controller&);
    void (*onDisconnect)(void* userdata, const Controller&);
    void (*onExit)(void* userdata, const Controller&);
    void (*onFrame)(void* userdata, const Controller&);
    void (*onFocusGained)(void* userdata, const Controller&);
    void (*onFocusLost)(void* userdata, const Controller&);
    void (*onServiceConnect)(void* userdata, const Controller&);
    void (*onServiceDisconnect)(void* userdata, const Controller&);
    void (*onDeviceChange)(void* userdata, const Controller&);
    void (*onImages)(void* userdata, const Controller&);

    void* userdata;
};

class RustListener: public Listener {
    RustListenerCallbacks m_callbacks;
public:
    RustListener(RustListenerCallbacks callbacks);

    void onInit(const Controller& c) override;

    void onConnect(const Controller& c) override;

    void onDisconnect(const Controller& c) override;

    void onExit(const Controller& c) override;

    void onFrame(const Controller& c) override;

    void onFocusGained(const Controller& c) override;

    void onFocusLost(const Controller& c) override;

    void onServiceConnect(const Controller& c) override;

    void onServiceDisconnect(const Controller& c) override;

    void onDeviceChange(const Controller& c) override;

    void onImages(const Controller& c) override;
};

Image RustGetImage(const ImageList& list, int index);

}
