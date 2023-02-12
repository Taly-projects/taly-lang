#include "main.h"

const char* ToCString_to_c_string(ToCString* self) { 
	((self->ToCString_to_c_string)(self));
}

String* String_create(const char* c_str) { 
	String* self = malloc(sizeof(String));
	((self->c_str) = c_str);
	return self;
}

void String_destroy(String* self) { 
	free(self);
}

const char* String_to_c_string_impl(String* self) { 
	return (self->c_str);
}

const char* String_to_c_string(String* self) { 
	String_to_c_string_impl(self);
}

int main() { 
	String* str = String_create("Hello");
	printf(String_to_c_string(str));
	String_destroy(str);
	return 0;
}

