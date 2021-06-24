#pragma once

typedef enum {
  u8,
  i8,
  u16,
  i16,
  u32,
  i32,
  f32,
  f64,
} ElementType;

typedef enum {
  Ok,
  IncorrectArgumentTypes,
  IncorrectArgumentSizes,
  InternalError,
} InferenceResult;

// A row-major N-dimensional tensor who's elements may be integers or floats
// of various bit-widths.
typedef struct {
  // What type of elements does this tensor contain?
  ElementType type;
  // Opaque bytes containing the tensor's data.
  void *data;
  // An array containing the length of each of the tensor's dimensions.
  int *shape;
  // How many dimensions are there?
  int rank;
} Tensor;

typedef struct Model Model;

// Load a model using its "mimetype" to figure out what format the model is
// in.
//
// Only "application/tflite-model" is accepted at this time.
Model *load(const char *mimetype, const void *model, int model_len);

// Run inference on
InferenceResult infer(Model *model, const Tensor *inputs, int num_inputs,
                      const Tensor *outputs, int num_outputs);
