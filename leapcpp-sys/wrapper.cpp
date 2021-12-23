#include "wrapper.hpp"

namespace Leap {

RustListener::RustListener(RustListenerCallbacks callbacks) : m_callbacks(callbacks) {}

void RustListener::onInit(const Controller& c) {
    if (m_callbacks.onInit) {
        m_callbacks.onInit(m_callbacks.userdata, c);
    }
}

void RustListener::onConnect(const Controller& c) {
    if (m_callbacks.onConnect) {
        m_callbacks.onConnect(m_callbacks.userdata, c);
    }
}

void RustListener::onDisconnect(const Controller& c) {
    if (m_callbacks.onDisconnect) {
        m_callbacks.onDisconnect(m_callbacks.userdata, c);
    }
}

void RustListener::onExit(const Controller& c) {
    if (m_callbacks.onExit) {
        m_callbacks.onExit(m_callbacks.userdata, c);
    }
}

void RustListener::onFrame(const Controller& c) {
    if (m_callbacks.onFrame) {
        m_callbacks.onFrame(m_callbacks.userdata, c);
    }
}

void RustListener::onFocusGained(const Controller& c) {
    if (m_callbacks.onFocusGained) {
        m_callbacks.onFocusGained(m_callbacks.userdata, c);
    }
}

void RustListener::onFocusLost(const Controller& c) {
    if (m_callbacks.onFocusLost) {
        m_callbacks.onFocusLost(m_callbacks.userdata, c);
    }
}

void RustListener::onServiceConnect(const Controller& c) {
    if (m_callbacks.onServiceConnect) {
        m_callbacks.onServiceConnect(m_callbacks.userdata, c);
    }
}

void RustListener::onServiceDisconnect(const Controller& c) {
    if (m_callbacks.onServiceDisconnect) {
        m_callbacks.onServiceDisconnect(m_callbacks.userdata, c);
    }
}

void RustListener::onDeviceChange(const Controller& c) {
    if (m_callbacks.onDeviceChange) {
        m_callbacks.onDeviceChange(m_callbacks.userdata, c);
    }
}

void RustListener::onImages(const Controller& c) {
    if (m_callbacks.onImages) {
        m_callbacks.onImages(m_callbacks.userdata, c);
    }
}

Image RustGetImage(const ImageList& list, int index) {
    return list[index];
}

}
