#pragma once

#include <stddef.h>

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

// Load a model using its "mimetype" to figure out what format the model is in
// Also tries to create an appropriate interpreter to execute the model
// Only "application/tflite-model" is accepted at this time.
Model *load(const char *mimetype, const void *model, int model_len);

// frees all the resources allocated for a model
void unload(Model *model);

// Run inference on the model with the inputs provided and collect the outputs
InferenceResult infer(Model *model, const Tensor *inputs, size_t num_inputs,
                      const Tensor *outputs, size_t num_outputs);
