extern "C" {
    #include "runecoral.h"
}

#include <cstring>
#include <vector>
#include <algorithm>

#include "coral/tflite_utils.h"
#include "tensorflow/lite/c/common.h"
#include "tensorflow/lite/interpreter.h"

RuneCoralLoadResult compare_tensors(const RuneCoralTensor &runeTensor, const TfLiteTensor &tfLiteTensor) {
    if (static_cast<int>(runeTensor.type) != static_cast<int>(tfLiteTensor.type)) {
        return RuneCoralLoadResult__IncorrectArgumentTypes;
    }

    if (runeTensor.rank != tfLiteTensor.dims->size) {
        return RuneCoralLoadResult__IncorrectArgumentSizes;
    }

    for (int i = 0; i < runeTensor.rank; i ++) {
        if (runeTensor.shape[i] != tfLiteTensor.dims->data[i]) {
            return RuneCoralLoadResult__IncorrectArgumentSizes;
        }
    }

    return RuneCoralLoadResult__Ok;
}

struct RuneCoralContext {
    std::unique_ptr<tflite::FlatBufferModel> model;
    std::shared_ptr<edgetpu::EdgeTpuContext> edgetpu_context;
    std::unique_ptr<tflite::Interpreter> interpreter;
};

RuneCoralLoadResult create_inference_context(const char *mimetype, const void *model, int model_len,
                                             const RuneCoralTensor *inputs, size_t num_inputs,
                                             const RuneCoralTensor *outputs, size_t num_outputs,
                                             RuneCoralContext **inferenceContext) {
    if (strcmp(mimetype, RUNE_CORAL_MIME_TYPE__TFLITE) != 0) {
        return RuneCoralLoadResult__IncorrectMimeType;
    }

    if (inferenceContext == nullptr) {
        return RuneCoralLoadResult__InternalError;
    }

    RuneCoralLoadResult result = RuneCoralLoadResult__Ok;

    RuneCoralContext *context = new RuneCoralContext();

    //TODO: See if this can be improved with a 0 copy alternative
    context->model = tflite::FlatBufferModel::BuildFromBuffer(reinterpret_cast<const char*>(model), model_len);

    // TODO: What if the context doesn't contain edgetpu graph nodes?
    context->edgetpu_context = coral::ContainsEdgeTpuCustomOp(*(context->model))
                              ? coral::GetEdgeTpuContextOrDie()
                              : nullptr;

    context->interpreter = coral::MakeEdgeTpuInterpreterOrDie(*(context->model), context->edgetpu_context.get());

    if (context->interpreter->AllocateTensors() != kTfLiteOk) {
        result =   RuneCoralLoadResult__InternalError;
    }

    if(context->interpreter->inputs().size() != num_inputs || context->interpreter->outputs().size() != num_outputs) {
        result = RuneCoralLoadResult__IncorrectArgumentSizes;
    }

    if (result == RuneCoralLoadResult__Ok) {
        for (size_t i = 0; i < num_inputs; i++) {
            auto inputTensor = context->interpreter->input_tensor(i);
            result  = compare_tensors(inputs[i], *inputTensor);

            if (result != RuneCoralLoadResult__Ok) {
                break;
            }
        }
    }

    if (result == RuneCoralLoadResult__Ok) {
        for (size_t i = 0; i < num_outputs; i++) {
            auto outputTensor = context->interpreter->output_tensor(i);
            result = compare_tensors(outputs[i], *outputTensor);

            if (result != RuneCoralLoadResult__Ok) {
                break;
            }
        }
    }

    if (!(context->model && context->edgetpu_context && context->interpreter && result == RuneCoralLoadResult__Ok)) {
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

// Run inference on
RuneCoralInferenceResult infer(RuneCoralContext *context, const RuneCoralTensor *inputs, size_t num_inputs,
                               RuneCoralTensor *outputs, size_t num_outputs) {
    // Validity checks
    if (context == nullptr) {
        return RuneCoralInferenceResult__Error;
    }

    // Feed inputs to the interpreter
    for (size_t i = 0; i < num_inputs; i++) {
        auto tfTensor = coral::MutableTensorData<char>(*context->interpreter->input_tensor(i));
        const auto& input = inputs[i];
        std::copy(reinterpret_cast<char*>(input.data), reinterpret_cast<char*>(input.data) + tfTensor.size(),
                  tfTensor.data());
    }

    auto inferenceResult = context->interpreter->Invoke();
    if (inferenceResult == kTfLiteOk) {
        //Collect outputs
        for (size_t i = 0; i < num_outputs; i++) {
            auto tfTensor = coral::TensorData<char>(*context->interpreter->input_tensor(i));
            std::copy(tfTensor.begin(), tfTensor.end(),
                      reinterpret_cast<char*>(outputs[i].data));
        }
        return RuneCoralInferenceResult__Ok;
    }

    return static_cast<RuneCoralInferenceResult>(inferenceResult);
}