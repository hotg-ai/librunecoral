extern "C" {
    #include "runecoral.h"
}

#include <cstring>
#include <vector>
#include <algorithm>
#include <iostream>

#include "tensorflow/lite/interpreter.h"
#include "tensorflow/lite/kernels/register.h"
#include "tensorflow/lite/model_builder.h"
#include "tflite/public/edgetpu_c.h"

#define RUNECORAL_ENABLE_LOGGING
#ifdef RUNECORAL_ENABLE_LOGGING
#define LOG_E(x)  {  std::cerr << "[runecoral] " << x << std::endl; }
#define LOG_D(x)  {  std::cerr << "[runecoral] " << x << std::endl; }
#else
#define LOG_E(x)  // nothing
#define LOG_D(x)  // nothing
#endif

RuneCoralLoadResult compare_tensors(const RuneCoralTensor &runeTensor, const TfLiteTensor &tfLiteTensor) {
    if (static_cast<int>(runeTensor.type) != static_cast<int>(tfLiteTensor.type)) {
        LOG_E("Tensor types mismatch")
        return RuneCoralLoadResult__IncorrectArgumentTypes;
    }

    if (runeTensor.rank != tfLiteTensor.dims->size) {
        LOG_E("Tensor rank mismatch.");
        return RuneCoralLoadResult__IncorrectArgumentSizes;
    }

    for (int i = 0; i < runeTensor.rank; i ++) {
        if (runeTensor.shape[i] != tfLiteTensor.dims->data[i]) {
            LOG_E("Tensor shape mismatch");
            return RuneCoralLoadResult__IncorrectArgumentSizes;
        }
    }

    return RuneCoralLoadResult__Ok;
}

struct RuneCoralContext {
    std::unique_ptr<tflite::FlatBufferModel> model;
    tflite::ops::builtin::BuiltinOpResolver resolver;
    std::unique_ptr<tflite::Interpreter> interpreter;
    size_t edgetpu_device_count = 0;
    struct edgetpu_device* edgetpu_devices = nullptr;

    ~RuneCoralContext() {
        LOG_D("Cleaning up Edgetpu context");
        if (edgetpu_devices) {
            edgetpu_free_devices(edgetpu_devices);
            edgetpu_devices = nullptr;
        }
    }
};

RuneCoralLoadResult create_inference_context(const char *mimetype, const void *model, int model_len,
                                             const RuneCoralTensor *inputs, size_t num_inputs,
                                             const RuneCoralTensor *outputs, size_t num_outputs,
                                             RuneCoralContext **inferenceContext) {
    if (strcmp(mimetype, RUNE_CORAL_MIME_TYPE__TFLITE) != 0) {
        LOG_E("Invalid Tensor Mimetype");
        return RuneCoralLoadResult__IncorrectMimeType;
    }

    if (inferenceContext == nullptr) {
        return RuneCoralLoadResult__InternalError;
    }

// TODO: Directly try to use libedgetpu
// 1. Create tflite::FlatBufferModel which may contain edge TPU custom op.
//
// auto model =
//    tflite::FlatBufferModel::BuildFromFile(model_file_name.c_str());
//
// 2. Create tflite::Interpreter.
//
// tflite::ops::builtin::BuiltinOpResolver resolver;
// std::unique_ptr<tflite::Interpreter> interpreter;
// tflite::InterpreterBuilder(model, resolver)(&interpreter);
//
// 3. Enumerate edge TPU devices.
//
// size_t num_devices;
// std::unique_ptr<edgetpu_device, decltype(&edgetpu_free_devices)> devices(
//     edgetpu_list_devices(&num_devices), &edgetpu_free_devices);
//
// assert(num_devices > 0);
// const auto& device = devices.get()[0];
//
// 4. Modify interpreter with the delegate.
//
// auto* delegate =
//     edgetpu_create_delegate(device.type, device.path, nullptr, 0);
// interpreter->ModifyGraphWithDelegate({delegate, edgetpu_free_delegate});
//
// 5. Prepare input tensors and run inference.
//
    RuneCoralLoadResult result = RuneCoralLoadResult__Ok;

    RuneCoralContext *context = new RuneCoralContext();

    //TODO: See if this can be improved with a 0 copy alternative
    context->model = tflite::FlatBufferModel::BuildFromBuffer(reinterpret_cast<const char*>(model), model_len);

    // Create the interpreter
    tflite::InterpreterBuilder(*(context->model), context->resolver)(&(context->interpreter));
    if (!context->interpreter) {
        LOG_E("Interpreter not ready");
        result = RuneCoralLoadResult__InternalError;
    } else {
        if (context->interpreter->AllocateTensors() != kTfLiteOk) {
            LOG_E("Interpreter unable to allocate tensors");
            result = RuneCoralLoadResult__InternalError;
        } else {

            context->edgetpu_devices = edgetpu_list_devices(&(context->edgetpu_device_count));

            if (context->edgetpu_device_count > 0) {
                LOG_D("Edgetpu devices found. Trying to Update the interpreter to use the delegate.");
                const auto& device = context->edgetpu_devices[0];
                TfLiteDelegate* delegate = edgetpu_create_delegate(device.type, device.path, nullptr, 0);
                context->interpreter->ModifyGraphWithDelegate(std::unique_ptr<TfLiteDelegate, decltype(&edgetpu_free_delegate)>(delegate, &edgetpu_free_delegate));
            }

            if (context->interpreter->inputs().size() != num_inputs || context->interpreter->outputs().size() != num_outputs) {
                LOG_E("Interpreter inputs/outputs do not match the number of inputs/outputs passed");
                result = RuneCoralLoadResult__IncorrectArgumentSizes;
            }
        }
    }

    // Validate the input tensors of the interpreter
    if (result == RuneCoralLoadResult__Ok) {
        for (size_t i = 0; i < num_inputs; i++) {
            auto inputTensor = context->interpreter->input_tensor(i);
            result  = compare_tensors(inputs[i], *inputTensor);

            if (result != RuneCoralLoadResult__Ok) {
                LOG_E("Input tensor mismatch");
                break;
            }
        }
    }

    // Validate the output tensors of the interpreter
    if (result == RuneCoralLoadResult__Ok) {
        for (size_t i = 0; i < num_outputs; i++) {
            auto outputTensor = context->interpreter->output_tensor(i);
            result = compare_tensors(outputs[i], *outputTensor);

            if (result != RuneCoralLoadResult__Ok) {
                LOG_E("Output tensor mismatch");
                break;
            }
        }
    }

    if (result != RuneCoralLoadResult__Ok) {
        delete context;
        context = nullptr;
        *inferenceContext = nullptr;
    } else {
        *inferenceContext = context;
    }

    return result;
}

void destroy_inference_context(RuneCoralContext **context) {
    if (context && *context) {
        delete *context;
        *context = nullptr;
    }
}

RuneCoralInferenceResult infer(RuneCoralContext *context, const RuneCoralTensor *inputs, size_t num_inputs,
                               RuneCoralTensor *outputs, size_t num_outputs) {
    // Validity checks
    if (context == nullptr) {
        return RuneCoralInferenceResult__Error;
    }

    // Feed inputs to the interpreter
    for (size_t i = 0; i < num_inputs; i++) {
        auto tfTensor = context->interpreter->input_tensor(i);
        const auto& input = inputs[i];
        std::copy(reinterpret_cast<char*>(input.data), reinterpret_cast<char*>(input.data) + tfTensor->bytes,
                  reinterpret_cast<char*>(tfTensor->data.data));
    }

    auto inferenceResult = context->interpreter->Invoke();
    if (inferenceResult == kTfLiteOk) {
        //Collect outputs
        for (size_t i = 0; i < num_outputs; i++) {
            auto tfTensor = context->interpreter->output_tensor(i);
            std::copy(reinterpret_cast<char*>(tfTensor->data.data), reinterpret_cast<char*>(tfTensor->data.data) + tfTensor->bytes,
                      reinterpret_cast<char*>(outputs[i].data));
        }
        return RuneCoralInferenceResult__Ok;
    }

    return static_cast<RuneCoralInferenceResult>(inferenceResult);
}