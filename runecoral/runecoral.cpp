extern "C" {
    #include "runecoral.h"
}

#include <string>
#include <vector>
#include <algorithm>

#include "coral/tflite_utils.h"
#include "tensorflow/lite/c/common.h"
#include "tensorflow/lite/interpreter.h"


static const std::string TFLITE_MIME_TYPE = "application/tflite-model";

InferenceResult compare_tensors(const Tensor &runeTensor, const TfLiteTensor &tfLiteTensor) {
    // FIXME: Missing tflite types in ElementType: Fix me.
    //   kTfLiteNoType = 0,
    //   kTfLiteInt64 = 4,
    //   kTfLiteString = 5,
    //   kTfLiteBool = 6,
    //   kTfLiteComplex64 = 8,
    //   kTfLiteComplex128 = 12,
    // Once we sync up the types, the comparison could simply be a int cast
    // Missing ElementType in TFLiteType
    // u16, u32
    ElementType e = runeTensor.type;
    TfLiteType t = tfLiteTensor.type;
    if (!((e == u8 && t == kTfLiteUInt8) || (e == i8 && t == kTfLiteInt8)
         || (e == i16 && t == kTfLiteInt16)
         || (e == i32 && t == kTfLiteInt32)
         || (e == f32 && t == kTfLiteFloat32) || (e == f64 && t == kTfLiteFloat64))) {
        return IncorrectArgumentTypes;
    }

    if (runeTensor.rank != tfLiteTensor.dims->size) {
        return IncorrectArgumentSizes;
    }

    for (int i = 0; i < runeTensor.rank; i ++) {
        if (runeTensor.shape[i] != tfLiteTensor.dims->data[i]) {
            return IncorrectArgumentSizes;
        }
    }

    return Ok;
}

struct Model {
    std::unique_ptr<tflite::FlatBufferModel> model;
    std::shared_ptr<edgetpu::EdgeTpuContext> edgetpu_context;
    std::unique_ptr<tflite::Interpreter> interpreter;
};

Model *load(const char *mimetype, const void *model, int model_len) {
    if (mimetype != TFLITE_MIME_TYPE) {
        return nullptr;
    }

    auto result = new Model();

    //TODO: See if this can be improved with a 0 copy alternative
    result->model = tflite::FlatBufferModel::BuildFromBuffer(reinterpret_cast<const char*>(model), model_len);

    // TODO: What if the model doesn't contain edgetpu graph nodes?
    result->edgetpu_context = coral::ContainsEdgeTpuCustomOp(*(result->model))
                              ? coral::GetEdgeTpuContextOrDie()
                              : nullptr;

    result->interpreter = coral::MakeEdgeTpuInterpreterOrDie(*(result->model), result->edgetpu_context.get());

    bool tensors_allocated = (result->interpreter->AllocateTensors() == kTfLiteOk);

    if (!(result->model && result->edgetpu_context && result->interpreter && tensors_allocated)) {
        delete result;
        result = nullptr;
    }

    return result;
}

void unload(Model *model) {
    if (model) {
        delete model;
    }
}

// Run inference on
InferenceResult infer(Model *model, const Tensor *inputs, size_t num_inputs,
                      const Tensor *outputs, size_t num_outputs) {
    // Validity checks
    if (model == nullptr) {
        return InternalError;
    }

    // TODO: These validity checks can be moved to a separate function and not be computed every invocation
    if (model->interpreter->inputs().size() != num_inputs || model->interpreter->outputs().size() != num_outputs) {
        return IncorrectArgumentSizes;
    }

    for (size_t i = 0; i < num_inputs; i++) {
        auto inputTensor = model->interpreter->input_tensor(i);
        auto tensorComparisonResult  = compare_tensors(inputs[i], *inputTensor);

        if (tensorComparisonResult != Ok) {
            return tensorComparisonResult;
        }
    }

    for (size_t i = 0; i < num_outputs; i++) {
        auto outputTensor = model->interpreter->output_tensor(i);
        auto tensorComparisonResult = compare_tensors(outputs[i], *outputTensor);

        if (tensorComparisonResult != Ok) {
            return tensorComparisonResult;
        }
    }

    // Feed inputs to the interpreter
    for (size_t i = 0; i < num_inputs; i++) {
        auto tfTensor = coral::MutableTensorData<char>(*model->interpreter->input_tensor(i));
        const auto& input = inputs[i];
        std::copy(reinterpret_cast<char*>(input.data), reinterpret_cast<char*>(input.data) + tfTensor.size(),
                  tfTensor.data());
    }

    if (model->interpreter->Invoke() == kTfLiteOk) {
        //Collect outputs
        for (size_t i = 0; i < num_outputs; i++) {
            auto tfTensor = coral::MutableTensorData<char>(*model->interpreter->input_tensor(i));
            std::copy(tfTensor.begin(), tfTensor.end(),
                      reinterpret_cast<char*>(outputs[i].data));
        }
        return Ok;
    }

    return InternalError;
}