#include <iostream>
#include <sinemodel.h>
#include <vector>

extern "C" {
#include <runecoral.h>
}

int main(int argc, char* argv[]) {
    std::vector<float> inputData = {0.8};
    std::vector<float> outputData = {0};
    std::vector<int> tensorShape = {1,1};

    RuneCoralContext  *context = nullptr;

    RuneCoralTensor inputs[] = { {RuneCoralElementType__Float32, inputData.data(), tensorShape.data(), static_cast<int>(tensorShape.size())} };
    RuneCoralTensor outputs[] = { {RuneCoralElementType__Float32, outputData.data(), tensorShape.data(), static_cast<int>(tensorShape.size())} };

    auto contextCreationResult = create_inference_context(RUNE_CORAL_MIME_TYPE__TFLITE,
                                                          reinterpret_cast<const void*>(Resources::sinemodel_tflite), Resources::sinemodel_tflite_size,
                                                          inputs, 1,  outputs, 1, &context);

    if (contextCreationResult == RuneCoralLoadResult__Ok) {
        std::cout << "Created context for inference" << std::endl;
        auto result = infer(context, inputs, 1, outputs, 1);
        if (result == RuneCoralInferenceResult__Ok) {
            std::cout << "Inference result = " << outputData[0] << std::endl;
        } else {
            std::cerr << "Inference failed with error code: " << static_cast<int>(result) << std::endl;
        }

    } else {
        std::cerr << "Error Creating context: " << static_cast<int>(contextCreationResult) << std::endl;
    }

    destroy_inference_context(&context);
    return 0;
}
