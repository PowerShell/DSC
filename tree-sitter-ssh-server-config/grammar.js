//-------------------------------------------------------------------------------------------------------
// Copyright (C) Microsoft. All rights reserved.
// Licensed under the MIT license. See LICENSE file in the project root for full license information.
//-------------------------------------------------------------------------------------------------------

const PREC = {
  MATCH: 2,
  OPERATOR: 1,
}

module.exports = grammar({
  name: 'ssh_server_config',

  extras: $ => [' ', '\t', '\r'],

  rules: {
    server_config: $ => seq(repeat(choice($.empty_line, $.comment, $.keyword)), repeat($.match)),

    // check for an empty line that is just a /n character
    empty_line: $ => seq('\n'),
    comment: $ => /#.*\n/,

    keyword: $ => seq(
      field('keyword', $.alphanumeric),
      choice(seq(/[ \t]/, optional('=')), '='),
      optional(field('operator', $.operator)),
      field('arguments', $.arguments),
      "\n"
    ),

    match: $ => seq(
      token(prec(PREC.MATCH, /match/i)),
      field('criteria', $.keyword),
      repeat1(choice($.comment, $.keyword)),
    ),

    arguments: $ => repeat1(choice($.boolean, $.number, $._quotedString, $._commaSeparatedString)),

    alphanumeric: $ => /[a-zA-Z0-9]+/i,
    boolean: $ => choice('yes', 'no'),
    number: $ => /\d+/,
    operator: $ => token(prec(PREC.OPERATOR, /[-+\^]/)),
    string: $ => /[^\r\n,"]+/,

    _commaSeparatedString: $ => seq($.string, repeat(seq(',', $.string))),
    _quotedString: $ => seq('\"', $.string, '\"'),
  }

});
