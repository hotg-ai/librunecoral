#pragma once

#include <stddef.h>

const char *RUNE_CORAL_MIME_TYPE__TFLITE = "application/tflite-model";

// These types match the TfLiteType from tensorflow.
// When modifying these types, only add to this enum
typedef enum {
  RuneCoralElementType__NoType = 0,
  RuneCoralElementType__Float32 = 1,
  RuneCoralElementType__Int32 = 2,
  RuneCoralElementType__UInt8 = 3,
  RuneCoralElementType__Int64 = 4,
  RuneCoralElementType__String = 5,
  RuneCoralElementType__Bool = 6,
  RuneCoralElementType__Int16 = 7,
  RuneCoralElementType__Complex64 = 8,
  RuneCoralElementType__Int8 = 9,
  RuneCoralElementType__Float16 = 10,
  RuneCoralElementType__Float64 = 11,
  RuneCoralElementType__Complex128 = 12,
} RuneCoralElementType;

// TODO: Extend this to support the quantization type
// A row-major N-dimensional tensor who's elements may be integers or floats
// of various bit-widths.
typedef struct {
  // What type of elements does this tensor contain?
  RuneCoralElementType type;
  // Opaque bytes containing the tensor's data.
  void *data;
  // An array containing the length of each of the tensor's dimensions.
  const size_t *shape;
  // How many dimensions are there?
  size_t rank;
} RuneCoralTensor;

typedef struct RuneCoralContext RuneCoralContext;

typedef enum {
  RuneCoralLoadResult__Ok = 0,
  RuneCoralLoadResult__IncorrectMimeType,
  RuneCoralLoadResult__IncorrectArgumentTypes,
  RuneCoralLoadResult__IncorrectArgumentSizes,
  RuneCoralLoadResult__InternalError,
} RuneCoralLoadResult;

// Load a model using its "mimetype" to figure out what format the model is in
// Only "application/tflite-model" is accepted at this time.
// And then create an interpreter for the model to be interpreted
// Also verifies if the input and output tensors match that of model
RuneCoralLoadResult create_inference_context(const char *mimetype, const void *model, size_t model_len,
                                             const RuneCoralTensor *inputs, size_t num_inputs,
                                             const RuneCoralTensor *outputs, size_t num_outputs,
                                             RuneCoralContext **inferenceContext);

// frees all the resources allocated for a context
void destroy_inference_context(RuneCoralContext *inferenceContext);

// Modeled after TfLiteStatus for now
typedef enum {
  RuneCoralInferenceResult__Ok = 0,
  // Generally referring to an error in the runtime (i.e. interpreter)
  RuneCoralInferenceResult__Error = 1,
  // Generally referring to an error from a TfLiteDelegate itself.
  RuneCoralInferenceResult__DelegateError = 2,
  // Generally referring to an error in applying a delegate due to
  // incompatibility between runtime and delegate, e.g., this error is returned
  // when trying to apply a TfLite delegate onto a model graph that's already
  // immutable.
  RuneCoralInferenceResult__ApplicationError = 3
} RuneCoralInferenceResult;

// Run inference on the model with the inputs provided and collect the outputs
RuneCoralInferenceResult infer(RuneCoralContext *context,
                               const RuneCoralTensor *inputs, size_t num_inputs,
                               RuneCoralTensor *outputs, size_t num_outputs);
