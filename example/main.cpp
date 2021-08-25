#include <iostream>
#include <sinemodel.h>
#include <vector>
#include <cassert>
#include <cmath>

extern "C" {
#include <runecoral.h>
}

int main(int argc, char* argv[]) {
    std::vector<float> inputData = {0.8};
    std::vector<float> outputData = {0};
    std::vector<size_t> tensorShape = {1,1};

    RuneCoralContext  *context = nullptr;

    RuneCoralTensor inputs[] = { {RuneCoralElementType__Float32, inputData.data(), tensorShape.data(), tensorShape.size()} };
    RuneCoralTensor outputs[] = { {RuneCoralElementType__Float32, outputData.data() , tensorShape.data(), tensorShape.size()} };

    int backends = availableAccelerationBackends();

    std::cout << "Can use Tpu Backend? " << (backends & RuneCoralAccelerationBackend__Libedgetpu ? "yes" : "no") << std::endl;
    std::cout << "Can use Gpu Backend? " << (backends & RuneCoralAccelerationBackend__Gpu ? "yes" : "no") << std::endl;

    auto contextCreationResult = create_inference_context(RUNE_CORAL_MIME_TYPE__TFLITE,
                                                          reinterpret_cast<const void*>(Resources::sinemodel_tflite), Resources::sinemodel_tflite_size,
                                                          inputs, 1,  outputs, 1,
                                                          RuneCoralAccelerationBackend__None,
                                                          &context);

    if (contextCreationResult == RuneCoralLoadResult__Ok) {
        std::cout << "Created context for inference" << std::endl;
        auto result = infer(context, inputs, 1, outputs, 1);
        if (result == RuneCoralInferenceResult__Ok) {
            assert(std::fabs(outputData[0] -  0.697279f) < 0.00001);
            std::cout << "Inference result = " << outputData[0] << std::endl;
        } else {
            std::cerr << "Inference failed with error code: " << static_cast<int>(result) << std::endl;
        }

    } else {
        std::cerr << "Error Creating context: " << static_cast<int>(contextCreationResult) << std::endl;
    }

    destroy_inference_context(context);
    return 0;
}
