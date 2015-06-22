/*
 * Licensed to the Apache Software Foundation (ASF) under one
 * or more contributor license agreements. See the NOTICE file
 * distributed with this work for additional information
 * regarding copyright ownership. The ASF licenses this file
 * to you under the Apache License, Version 2.0 (the
 * "License"); you may not use this file except in compliance
 * with the License. You may obtain a copy of the License at
 *
 *   http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
 * KIND, either express or implied. See the License for the
 * specific language governing permissions and limitations
 * under the License.
 */

#include <map>
#include <fstream>
#include <sstream>
#include <string>
#include <vector>

#include "t_oop_generator.h"
#include "platform.h"
#include "version.h"
#include "logging.h"

using std::map;
using std::ofstream;
using std::string;
using std::vector;

/**
 * Rust code generator.
 */
class t_rs_generator : public t_oop_generator {
 public:
  t_rs_generator(t_program* program,
                 const map<string, string>& parsed_options,
                 const string& option_string)
    : t_oop_generator(program)
  {
    (void) parsed_options;
    (void) option_string;
    // FIXME: change back to gen-rs when we finalize mod structure for generated code
    out_dir_base_ = "src";
  }

  void init_generator();
  void close_generator();

  /**
   * Program-level generation functions
   */
  void generate_program();
  void generate_typedef(t_typedef*  ttypedef);
  void generate_enum(t_enum*     tenum);
  void generate_struct(t_struct*   tstruct);
  void generate_service(t_service*  tservice);

 private:
  string rs_autogen_comment();
  string rs_imports();

  string render_rs_type(t_type* type);
  string render_suffix(t_type* type);
  string render_type_init(t_type* type);

  void generate_service_generics(t_service* tservice);
  void generate_service_fields(t_service* tservice);
  void generate_service_methods(char field, t_service* tservice);
  void generate_service_method_arglist(const vector<t_field*>& fields);
  void generate_service_uses(t_service* tservice);

  /**
   *Transforms a string with words separated by underscores to a pascal case equivalent
   * e.g. a_multi_word -> AMultiWord
   *      some_name    ->  SomeName
   *      name         ->  Name
   */
  std::string pascalcase(const std::string& in) {
    return capitalize(camelcase(in));
  }

  bool is_string(t_type* type) {
    return type->is_string() && !((t_base_type*)type)->is_binary();
  }

  bool is_binary(t_type* type) {
    return type->is_string() && ((t_base_type*)type)->is_binary();
  }

  static bool is_keyword(const string& id) {
    static string keywords =
      "|abstract|alignof|as|be|box|break|const|continue|crate|do|else|enum|extern|false|final|"
      "fn|for|if|impl|in|let|loop|macro|match|mod|move|mut|offsetof|override|priv|pub|pure|ref|"
      "return|sizeof|static|self|struct|super|true|trait|type|typeof|unsafe|unsized|use|virtual|"
      "where|while|yield|";

    return keywords.find("|" + id + "|") != string::npos;
  }

  static string normalize_id(const string& id) {
    return is_keyword(id) ? id + "_" : id;
  }

  string to_field_name(const string& id) {
    return normalize_id(underscore(id));
  }

 private:
  ofstream f_mod_;
};

/*
 * Helper class for allocating temp variable names
 */
class t_temp_var {
public:
  t_temp_var() {
    std::stringstream ss;
    // FIXME: are we safe for name clashes?
    ss << "tmp" << index_++;
    name_ = ss.str();
  }
  ~t_temp_var() {
    --index_;
  }
  const string& str() const { return name_; }
private:
  static int index_;
  string name_;
};

int t_temp_var::index_ = 0;


/*
 * This is necessary because we want to generate use clauses for all services,
 */
void t_rs_generator::generate_program() {
  // Initialize the generator
  init_generator();

  // Generate service uses
  vector<t_service*> services = program_->get_services();
  vector<t_service*>::iterator sv_iter;
  for (sv_iter = services.begin(); sv_iter != services.end(); ++sv_iter) {
    generate_service_uses(*sv_iter);
  }

  // Generate enums
  vector<t_enum*> enums = program_->get_enums();
  vector<t_enum*>::iterator en_iter;
  for (en_iter = enums.begin(); en_iter != enums.end(); ++en_iter) {
    generate_enum(*en_iter);
  }

  // Generate typedefs
  vector<t_typedef*> typedefs = program_->get_typedefs();
  vector<t_typedef*>::iterator td_iter;
  for (td_iter = typedefs.begin(); td_iter != typedefs.end(); ++td_iter) {
    generate_typedef(*td_iter);
  }

  // Generate structs, exceptions, and unions in declared order
  vector<t_struct*> objects = program_->get_objects();
  vector<t_struct*>::iterator o_iter;
  for (o_iter = objects.begin(); o_iter != objects.end(); ++o_iter) {
    generate_struct(*o_iter);
  }

  // Generate constants
  // TODO: Implement constant generation.
  // vector<t_const*> consts = program_->get_consts();
  // generate_consts(consts);

  // Generate services
  for (sv_iter = services.begin(); sv_iter != services.end(); ++sv_iter) {
    generate_service(*sv_iter);
  }

  // Close the generator
  close_generator();
}

void t_rs_generator::init_generator() {
  // Make output directory
  // FIXME: enable when finalizing the code structure
  //MKDIR(get_out_dir().c_str());
  string pname = underscore(program_name_);
  string moddirname = get_out_dir() + pname + "/";
  MKDIR(moddirname.c_str());

  // Make output file
  string f_mod_name = moddirname + "mod.rs";
  f_mod_.open(f_mod_name.c_str());

  // Print header
  f_mod_ << rs_autogen_comment() << "\n";
  f_mod_ << rs_imports() << "\n";
}

void t_rs_generator::close_generator() {
  f_mod_.close();
}

string t_rs_generator::rs_autogen_comment() {
  return string(
    "///////////////////////////////////////////////////////////////\n") +
    "// Autogenerated by Thrift Compiler (" + THRIFT_VERSION + ")\n" +
    "//\n" +
    "// DO NOT EDIT UNLESS YOU ARE SURE YOU KNOW WHAT YOU ARE DOING\n" +
    "///////////////////////////////////////////////////////////////\n";
}

string t_rs_generator::rs_imports() {
  return string("#![allow(unused_mut, dead_code, non_snake_case)]\n") +
          "#[allow(unused_imports)]\n" +
          "use std::collections::{HashMap, HashSet};\n";
}

// Generates a type alias, translating a thrift `typedef` to a rust `type`.
void t_rs_generator::generate_typedef(t_typedef* ttypedef) {
  string tname = pascalcase(ttypedef->get_symbolic());
  string tdef = render_rs_type(ttypedef->get_type());
  indent(f_mod_) << "pub type " << tname << " = " << tdef << ";\n";
  f_mod_ << "\n";
}

// Generates an enum, translating a thrift enum into a rust enum.
void t_rs_generator::generate_enum(t_enum* tenum) {
  string ename = pascalcase(tenum->get_name());
  indent(f_mod_) << "enom! {\n";
  indent_up();

  indent(f_mod_) << "name = " << ename << ",\n";

  indent(f_mod_) << "values = [\n";
  indent_up();

  // Generate the enum variant declarations.
  vector<t_enum_value*> constants = tenum->get_constants();
  vector<t_enum_value*>::iterator i, end = constants.end();
  for (i = constants.begin(); i != end; ++i) {
    string name = capitalize((*i)->get_name());
    int value = (*i)->get_value();
    indent(f_mod_) << name << " = " << value << ",\n";
  }

  indent_down();
  indent(f_mod_) << "],\n";
  indent(f_mod_) << "default = " << capitalize(constants.at(0)->get_name()) << "\n";

  indent_down();
  indent(f_mod_) << "}\n\n"; // Close enom invocation.
}

// Generate a struct, translating a thrift struct into a rust struct.
void t_rs_generator::generate_struct(t_struct* tstruct) {
  string sname = pascalcase(tstruct->get_name());

  indent(f_mod_) << "strukt! {\n";
  indent_up();

  indent(f_mod_) << "name = " << sname << ",\n";

  indent(f_mod_) << "fields = {\n";
  indent_up();

  vector<t_field*>::const_iterator m_iter;
  const vector<t_field*>& members = tstruct->get_members();
  for (m_iter = members.begin(); m_iter != members.end(); ++m_iter) {
    t_field* tfield = *m_iter;
    indent(f_mod_) << to_field_name(tfield->get_name())
      << ": " << render_rs_type(tfield->get_type())
      << " => " << tfield->get_key() << ",\n";
  }

  indent_down();
  indent(f_mod_) << "}\n";

  indent_down();
  indent(f_mod_) << "}\n\n"; // Close strukt invocation.
}

// Generate a service, translating from a thrift service to a rust trait.
void t_rs_generator::generate_service(t_service* tservice) {
    const string sname = pascalcase(tservice->get_name());
    const string trait_name = sname;
    const string processor_name = sname + "Processor";
    const string client_name = sname + "Client";

    indent(f_mod_) << "service! {\n";
    indent_up();

    // Trait, processor and client type names.
    indent(f_mod_) << "trait_name = " << trait_name << ",\n";
    indent(f_mod_) << "processor_name = " << processor_name << ",\n";
    indent(f_mod_) << "client_name = " << client_name << ",\n";

    // The methods originating in this service to go in the service trait.
    indent(f_mod_) << "service_methods = [\n";
    indent_up();

    generate_service_methods('a', tservice);

    indent_down();
    indent(f_mod_) << "],\n";

    // The methods from parent services that need to go in the processor.
    indent(f_mod_) << "parent_methods = [\n";
    indent_up();

    char field;
    t_service* parent;
    for (parent = tservice->get_extends(), field = 'b';
         parent && field <= 'z';
         parent = parent->get_extends(), field++) {
        generate_service_methods(field, parent);
    }

    indent_down();
    indent(f_mod_) << "],\n";

    indent(f_mod_) << "bounds = [";
    generate_service_generics(tservice);
    f_mod_ << "],\n";

    indent(f_mod_) << "fields = [";
    generate_service_fields(tservice);
    f_mod_ << "]\n";

    indent_down();
    indent(f_mod_) << "}\n\n";
}

void t_rs_generator::generate_service_methods(char field, t_service* tservice) {
    const string sname = pascalcase(tservice->get_name());

    vector<t_function*> functions = tservice->get_functions();
    vector<t_function*>::const_iterator f_iter;
    for (f_iter = functions.begin(); f_iter != functions.end(); ++f_iter) {
        t_function* tfunction = *f_iter;
        const string argname = sname + pascalcase(tfunction->get_name()) + "Args";
        const string resname = sname + pascalcase(tfunction->get_name()) + "Result";

        indent(f_mod_) << argname << " -> " << resname << " = "
          << field << "." << tfunction->get_name() << "(\n";

        indent_up();
        generate_service_method_arglist(tfunction->get_arglist()->get_members());
        indent_down();

        indent(f_mod_) << ") -> " << render_rs_type(tfunction->get_returntype()) << " => [\n";

        indent_up();
        generate_service_method_arglist(tfunction->get_xceptions()->get_members());
        indent_down();

        indent(f_mod_) << "],\n";
    }
}

void t_rs_generator::generate_service_generics(t_service* tservice) {
  t_service* parent = tservice;
  char generic = 'A';

  while (parent && generic <= 'Z') {
    f_mod_ << generic << ": " << parent->get_name() << ", ";
    parent = parent->get_extends();
    generic++;
  }
}

void t_rs_generator::generate_service_fields(t_service* tservice) {
  t_service* parent = tservice;
  char generic = 'A';
  char field = 'a';

  while (parent && generic <= 'Z' && field <= 'z') {
    f_mod_ << field << ": " << generic << ", ";
    parent = parent->get_extends();
    generic++;
    field++;
  }
}

void t_rs_generator::generate_service_method_arglist(const vector<t_field*>& fields) {
    vector<t_field*>::const_iterator field_iter;
    for (field_iter = fields.begin(); field_iter != fields.end(); ++field_iter) {
        t_field* tfield = *field_iter;
        indent(f_mod_) << to_field_name(tfield->get_name())
            << ": " << render_rs_type(tfield->get_type())
            << " => " << tfield->get_key() << ",\n";
    }
}

void t_rs_generator::generate_service_uses(t_service* tservice) {
  t_service* service = tservice->get_extends();
  while (service) {
    indent(f_mod_) << "use " << service->get_program()->get_name() << "::*;\n";
    service = service->get_extends();
  }
  indent(f_mod_) << "\n";
}

// Renders a rust type representing the passed in type.
string t_rs_generator::render_rs_type(t_type* type) {
  type = get_true_type(type);

  if (type->is_base_type()) {
    t_base_type::t_base tbase = ((t_base_type*)type)->get_base();
    switch (tbase) {
    case t_base_type::TYPE_VOID:
      return "()";
    case t_base_type::TYPE_STRING:
      return (((t_base_type*)type)->is_binary() ? "Vec<u8>" : "String");
    case t_base_type::TYPE_BOOL:
      return "bool";
    case t_base_type::TYPE_BYTE:
      return "i8";
    case t_base_type::TYPE_I16:
      return "i16";
    case t_base_type::TYPE_I32:
      return "i32";
    case t_base_type::TYPE_I64:
      return "i64";
    case t_base_type::TYPE_DOUBLE:
      return "f64";
    }

  } else if (type->is_enum()) {
    return capitalize(((t_enum*)type)->get_name());

  } else if (type->is_struct() || type->is_xception()) {
    return capitalize(((t_struct*)type)->get_name());

  } else if (type->is_map()) {
    t_type* ktype = ((t_map*)type)->get_key_type();
    t_type* vtype = ((t_map*)type)->get_val_type();
    return "HashMap<" + render_rs_type(ktype) + ", " + render_rs_type(vtype) + ">";

  } else if (type->is_set()) {
    t_type* etype = ((t_set*)type)->get_elem_type();
    return "HashSet<" + render_rs_type(etype) + ">";

  } else if (type->is_list()) {
    t_type* etype = ((t_list*)type)->get_elem_type();
    return "Vec<" + render_rs_type(etype) + ">";

  } else {
    throw "INVALID TYPE IN type_to_enum: " + type->get_name();
  }
  return ""; // silence the compiler warning
}

THRIFT_REGISTER_GENERATOR(rs, "Rust", "")

