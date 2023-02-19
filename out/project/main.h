#ifndef TALY_GEN_C_main_H
#define TALY_GEN_C_main_H

#include <stdio.h>
#include <stdlib.h>

typedef struct ToCString { 
	const char*(*ToCString_to_c_string)(struct ToCString*);
} ToCString;

const char* ToCString_to_c_string(ToCString* self);

typedef struct String { 
	ToCString base_ToCString;
	const char* c_str;
} String;

String* String_create(const char* c_str);

void String_destroy(String* self);

const char* String_to_c_string_impl(String* self);

const char* String_to_c_string(String* self);

#endif // TALY_GEN_C_main_H