#pragma once

#include "utils.h"
#include "tensorflow/lite/interpreter.h"

#ifdef RUNECORAL_EDGETPU_BACKEND
#include "tflite/public/edgetpu_c.h"
#endif

#ifdef RUNECORAL_GPU_BACKEND
#include "tensorflow/lite/delegates/gpu/delegate.h"
#endif

class AccelerationBackend {
public:
    virtual ~AccelerationBackend();
    virtual bool isAvailable() const {
        return true;
    }
    virtual bool accelerate(tflite::Interpreter *interpreter) = 0;
};

#ifdef RUNECORAL_EDGETPU_BACKEND
class EdgetpuAccelerationBackend: public AccelerationBackend {
    size_t mEdgetpuDeviceCount = 0;
    struct edgetpu_device* mEdgetpuDevices = nullptr;
public:
    EdgetpuAccelerationBackend() {
        mEdgetpuDevices = edgetpu_list_devices(&(mEdgetpuDeviceCount));
    }

    ~EdgetpuAccelerationBackend() {
        if (mEdgetpuDevices) {
            LOG_D("Cleaning up Edgetpu context");
            edgetpu_free_devices(mEdgetpuDevices);
            mEdgetpuDevices = nullptr;
        }
    }

    bool isAvailable() const override {
        // TODO: Filter the devices based on evnironment variable preferences like USB vs. PCI
        return mEdgetpuDeviceCount > 0;
    }

    bool accelerate(tflite::Interpreter *interpreter) override {
        if (!isAvailable()) {
            return false;
        }

        LOG_D("Edgetpu devices found. Trying to Update the interpreter to use the delegate.");
        const auto& device = mEdgetpuDevices[0];
        TfLiteDelegate* delegate = edgetpu_create_delegate(device.type, device.path, nullptr, 0);
        return interpreter->ModifyGraphWithDelegate(std::unique_ptr<TfLiteDelegate, decltype(&edgetpu_free_delegate)>(delegate, &edgetpu_free_delegate)) == kTfLiteOk;
    }
};
#endif


#ifdef RUNECORAL_GPU_BACKEND
class GpuAccelerationBackend: public AccelerationBackend {
    struct TfLiteDelegate* mGpuDelegate = nullptr;
public:
    GpuAccelerationBackend() {
        mGpuDelegate = TfLiteGpuDelegateV2Create(/*default options=*/nullptr);
    }

    ~GpuAccelerationBackend() {
        if (mGpuDelegate) {
            TfLiteGpuDelegateV2Delete(mGpuDelegate);
        }
    }

    bool isAvailable() const override {
        return mGpuDelegate != nullptr;
    }

    bool accelerate(tflite::Interpreter *interpreter) override {
        if (!isAvailable()) {
            return false;
        }
        return interpreter->ModifyGraphWithDelegate(mGpuDelegate) == kTfLiteOk;
    }
};
#endif