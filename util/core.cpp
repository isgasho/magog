/* core.cpp

   Copyright (C) 2012 Risto Saarelma

   This program is free software: you can redistribute it and/or modify
   it under the terms of the GNU General Public License as published by
   the Free Software Foundation, either version 3 of the License, or
   (at your option) any later version.

   This program is distributed in the hope that it will be useful,
   but WITHOUT ANY WARRANTY; without even the implied warranty of
   MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
   GNU General Public License for more details.

   You should have received a copy of the GNU General Public License
   along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

#include "core.hpp"
#include <cstdio>
#include <cstdarg>
#include <stdlib.h>
#ifdef __WIN32__
#include <windows.h>
#endif

void die(const char* str) {
  #ifdef __WIN32__
  MessageBox(NULL, str, "Error", MB_OK);
  #else
  fprintf(stderr, "%s\n", str);
  #endif
  exit(1);
}

size_t hash(const char* s) {
  size_t next = *s ? hash(s + 1) : 2166136261u;
  return (next ^ *s) * 16777619u;
}
