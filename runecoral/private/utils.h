#pragma once

#include <iostream>

#ifdef RUNECORAL_ENABLE_LOGGING
#define LOG_E(x)  {  std::cerr << "[runecoral] " << x << std::endl; }
#define LOG_D(x)  {  std::cerr << "[runecoral] " << x << std::endl; }
#else
#define LOG_E(x)  // nothing
#define LOG_D(x)  // nothing
#endif
