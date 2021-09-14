extern "C" {
    #include "runecoral.h"
}

#include <cstring>
#include <vector>
#include <algorithm>

#include "tensorflow/lite/interpreter.h"
#include "tensorflow/lite/kernels/register.h"
#include "tensorflow/lite/model_builder.h"

#include "private/accelerationbackends.h"
#include "private/utils.h"

const char* RUNE_CORAL_MIME_TYPE__TFLITE = "application/tflite-model";

RuneCoralTensor to_runecoraltensor(const TfLiteTensor &tfLiteTensor) {
    RuneCoralTensor result;
    result.data = nullptr;
    result.type = static_cast<RuneCoralElementType>(tfLiteTensor.type);
    result.rank = tfLiteTensor.dims->size;
    result.shape = tfLiteTensor.dims->data;
    return result;
}

struct RuneCoralContext {
    std::vector<char> model_buffer;
    // Note: model has pointers into model_buffer
    std::unique_ptr<tflite::FlatBufferModel> model;
    tflite::ops::builtin::BuiltinOpResolver resolver;
    std::unique_ptr<tflite::Interpreter> interpreter;
    std::unique_ptr<AccelerationBackend> accelerationBackend;
    std::vector<RuneCoralTensor> inputs;
    std::vector<RuneCoralTensor> outputs;

    // TODO: See if we can avoid this copy by keeping a reference to the
    // original model data
    RuneCoralContext(const char *model, size_t model_len)
        : model_buffer(model, model + model_len) {}
};

int availableAccelerationBackends() {
    int result = RuneCoralAccelerationBackend__None;
    //TODO : Add Runtime checks to this too
#ifdef RUNECORAL_EDGETPU_ACCELERATION
    EdgetpuAccelerationBackend edgeTpuBackend;
    if (edgeTpuBackend.isAvailable()) {
        result |= RuneCoralAccelerationBackend__Edgetpu;
    }
#endif

#ifdef RUNECORAL_GPU_ACCELERATION
    result |= RuneCoralAccelerationBackend__Gpu;
#endif

    return result;
}

bool accelerateInterpreter(const RuneCoralAccelerationBackend backend, RuneCoralContext *context) {
#ifdef RUNECORAL_EDGETPU_ACCELERATION
    if (backend & RuneCoralAccelerationBackend__Edgetpu) {
        context->accelerationBackend.reset(new EdgetpuAccelerationBackend());
    }
#endif

#ifdef RUNECORAL_GPU_ACCELERATION
    if (backend & RuneCoralAccelerationBackend__Gpu) {
        context->accelerationBackend.reset(new GpuAccelerationBackend());
    }
#endif

    return backend == RuneCoralAccelerationBackend__None
           || (context->accelerationBackend && context->accelerationBackend->accelerate(context->interpreter.get()));
}

RuneCoralLoadResult create_inference_context(const char *mimetype, const void *model, size_t model_len,
                                             const RuneCoralAccelerationBackend backend,
                                             RuneCoralContext **inferenceContext) {
    if (strcmp(mimetype, RUNE_CORAL_MIME_TYPE__TFLITE) != 0) {
        LOG_E("Invalid Tensor Mimetype");
        return RuneCoralLoadResult__IncorrectMimeType;
    }

    if (!(model && inferenceContext)) {
        return RuneCoralLoadResult__InternalError;
    }

    RuneCoralLoadResult result = RuneCoralLoadResult__Ok;

    RuneCoralContext *context = new RuneCoralContext{(const char *)model, model_len};

    context->model = tflite::FlatBufferModel::VerifyAndBuildFromBuffer(
        context->model_buffer.data(),
        context->model_buffer.size()
    );

    // Create the interpreter
    if (context->model) {
        tflite::InterpreterBuilder(*(context->model), context->resolver)(&(context->interpreter));

        if (context->interpreter) {
            if (!accelerateInterpreter(backend, context)) {
                LOG_E("Unable to accelerate interpreter");
            }

            if (context->interpreter->AllocateTensors() != kTfLiteOk) {
                LOG_E("Interpreter unable to allocate tensors");
                result = RuneCoralLoadResult__InternalError;
            } else {

                for (size_t i = 0; i < context->interpreter->inputs().size(); i++) {
                    context->inputs.push_back(to_runecoraltensor(*context->interpreter->input_tensor(i)));
                }

                for (size_t i = 0; i < context->interpreter->outputs().size(); i++) {
                    context->outputs.push_back(to_runecoraltensor(*context->interpreter->output_tensor(i)));
                }
            }
        } else {
            LOG_E("Interpreter not ready");
            result = RuneCoralLoadResult__InternalError;
        }
    } else {
        LOG_E("Unable to create a TFlite Model from the buffer that is passed");
        result = RuneCoralLoadResult__InternalError;
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

size_t inference_opcount(const RuneCoralContext * const inferenceContext) {
    if (!inferenceContext) {
        return 0;
    }

    size_t result = 0;
    for (const auto* subgraph : *(inferenceContext->model->GetModel())->subgraphs()) {
        for (const auto* op : *subgraph->operators()) {
            result++;
        }
    }

    return result;
}

size_t inference_inputs(const RuneCoralContext * const inferenceContext, const RuneCoralTensor ** tensors) {
    if (!inferenceContext) {
        *tensors = nullptr;
        return 0;
    }

    *tensors = inferenceContext->inputs.data();
    return inferenceContext->inputs.size();
}

size_t inference_outputs(const RuneCoralContext * const inferenceContext, const RuneCoralTensor ** tensors) {
    if (!inferenceContext) {
        *tensors = nullptr;
        return 0;
    }

    *tensors = inferenceContext->outputs.data();
    return inferenceContext->outputs.size();
}

void destroy_inference_context(RuneCoralContext *context) {
    delete context;
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
