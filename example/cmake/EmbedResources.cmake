# Creates C++ resources file from files in given directory
function(embed_resources output resources)
    # Create empty output file
    file(WRITE ${output} "#include <cstdlib>\n#include <cwchar>\n\nnamespace Resources {\n\n")

    # Iterate through input files
    foreach(bin ${resources})
        message("-- Creating a resource for : " ${bin})
        # Get short filename
        string(REGEX MATCH "([^/]+)$" filename ${bin})
        # Replace filename spaces & extension separator for C compatibility
        string(REGEX REPLACE "\\.| |-" "_" filename ${filename})
        # Read hex data from file
        file(READ ${bin} filedata HEX)
        # Convert hex data for C compatibility
        string(REGEX REPLACE "([0-9a-f][0-9a-f])" "0x\\1," filedata ${filedata})
        string(REGEX REPLACE ",$" "" filedata ${filedata})
        # Append data to output file
        file(APPEND ${output} "const unsigned char ${filename}[] = {${filedata}};\nconst size_t ${filename}_size = sizeof(${filename});\n")
    endforeach()

    file(APPEND ${output} "\n};")
endfunction()
