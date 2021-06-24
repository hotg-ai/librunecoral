extern "C" {
    #include "runecoral.h"
}

#include <iostream>
#include <string>
#include <cstdio>
#include <fstream>
#include <vector>

#include "absl/flags/flag.h"
#include "absl/flags/parse.h"
#include "coral/classification/adapter.h"
#include "coral/tflite_utils.h"
#include "tensorflow/lite/interpreter.h"
#include "absl/strings/ascii.h"
#include "absl/strings/numbers.h"
#include "absl/strings/str_split.h"
#include "glog/logging.h"

Model *load(const char *mimetype, const void *model, int model_len) {
    return nullptr;
}

// Run inference on
InferenceResult infer(Model *model, const Tensor *inputs, int num_inputs,
                      const Tensor *outputs, int num_outputs) {
    return Ok;
}

ABSL_FLAG(std::string, model_path, "mobilenet_v1_1.0_224_quant_edgetpu.tflite",
          "Path to the tflite model.");
ABSL_FLAG(std::string, image_path, "cat.rgb",
          "Path to the image to be classified. The input image size must match "
          "the input size of the model and the image must be stored as RGB "
          "pixel array.");
ABSL_FLAG(std::string, labels_path, "imagenet_labels.txt",
          "Path to the imagenet labels.");

namespace coral { // Coral examples' file utils because it is private by default
    std::unordered_map<int, std::string> ReadLabelFile(const std::string& file_path) {
        std::unordered_map<int, std::string> labels;
        std::ifstream file(file_path.c_str());
        CHECK(file) << "Cannot open " << file_path;

        std::string line;
        while (std::getline(file, line)) {
            absl::RemoveExtraAsciiWhitespace(&line);
            std::vector<std::string> fields = absl::StrSplit(line, absl::MaxSplits(' ', 1));
            if (fields.size() == 2) {
                int label_id;
                CHECK(absl::SimpleAtoi(fields[0], &label_id));
                const std::string& label_name = fields[1];
                labels[label_id] = label_name;
            }
        }

        return labels;
    }
    
    void ReadFileToOrDie(const std::string& file_path, char* data, size_t size) {
        std::ifstream file(file_path, std::ios::binary);
        CHECK(file) << "Cannot open " << file_path;
        CHECK(file.read(data, size))
            << "Cannot read " << size << " bytes from " << file_path;
        CHECK_EQ(file.peek(), EOF)
            << file_path << " size must match input size of " << size << " bytes";
    }
}

int sampleFunctionToTestLinking(int argc, char* argv[]) {
  absl::ParseCommandLine(argc, argv);

  // Load the model.
  const auto model = coral::LoadModelOrDie(absl::GetFlag(FLAGS_model_path));
  auto edgetpu_context = coral::ContainsEdgeTpuCustomOp(*model)
                             ? coral::GetEdgeTpuContextOrDie()
                             : nullptr;
  auto interpreter = coral::MakeEdgeTpuInterpreterOrDie(*model, edgetpu_context.get());
  CHECK_EQ(interpreter->AllocateTensors(), kTfLiteOk);

  // Read the image to input tensor.
  auto input = coral::MutableTensorData<char>(*interpreter->input_tensor(0));
  coral::ReadFileToOrDie(absl::GetFlag(FLAGS_image_path), input.data(), input.size());
  CHECK_EQ(interpreter->Invoke(), kTfLiteOk);

  // Read the label file.
  auto labels = coral::ReadLabelFile(absl::GetFlag(FLAGS_labels_path));

  for (auto result : coral::GetClassificationResults(*interpreter, 0.0f, /*top_k=*/3)) {
    std::cout << "---------------------------" << std::endl;
    std::cout << labels[result.id] << std::endl;
    std::cout << "Score: " << result.score << std::endl;
  }
  return 0;
}
