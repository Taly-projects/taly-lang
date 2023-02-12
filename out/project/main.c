#include "main.h"

const char* ToCString_to_c_string(ToCString* self) { 
	((self->ToCString_to_c_string)(self));
}

String* String_create_impl(const char* c_str) { 
	String* self = malloc(sizeof(String));
	((self->c_str) = c_str);
	return self;
}

String* String_create(const char* c_str) { 
	String_create_impl(c_str);
}

const char* String_to_c_string_impl(String* self) { 
	return (self->c_str);
}

const char* String_to_c_string(String* self) { 
	String_to_c_string_impl(self);
}

void String_destroy_impl(String* self) { 
	free(self);
}

void String_destroy(String* self) { 
	String_destroy_impl(self);
}

int main() { 
	String* str = String_create("Hello");
	printf(String_to_c_string(str));
	return 0;
}

