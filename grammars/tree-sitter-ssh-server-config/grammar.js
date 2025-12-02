//-------------------------------------------------------------------------------------------------------
// Copyright (C) Microsoft. All rights reserved.
// Licensed under the MIT license. See LICENSE file in the project root for full license information.
//-------------------------------------------------------------------------------------------------------

const PREC = {
  MATCH: 2,
  OPERATOR: 1
}

module.exports = grammar({
  name: 'ssh_server_config',

  extras: $ => [' ', '\t', '\r'],

  rules: {
    server_config: $ => seq(repeat(choice($._empty_line, $.comment, $.keyword)), repeat($.match)),

    // check for an empty line that is just a /n character
    _empty_line: $ => '\n',
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
      seq(repeat1($.criteria), $._empty_line),
      repeat1(choice($.comment, $.keyword)),
    ),

    criteria: $ => seq(
      field('criteria', $.alpha),
      choice(seq(/[ \t]/, optional('=')), '='),
      field('argument', $._argument)
    ),

    _argument: $ => choice($.boolean, $.number, $.string, $._commaSeparatedString, $._doublequotedString, $._singlequotedString),
    arguments: $ => repeat1($._argument),

    alpha: $ => /[a-zA-Z]+/i,
    alphanumeric: $ => /[a-zA-Z0-9]+/i,
    boolean: $ => choice('yes', 'no'),
    number: $ => /\d+/,
    operator: $ => token(prec(PREC.OPERATOR, /[-+\^]/)),
    string: $ => /[^\r\n,"'\s]+/, /* cannot contain spaces */

    _quotedString: $ => /[^\r\n,"']+/, /* can contain spaces */
    _doublequotedString: $ => seq('"', alias($._quotedString, $.string), repeat(seq(',', alias($._quotedString, $.string))), '"'),
    _singlequotedString: $ => seq('\'', alias($._quotedString, $.string), repeat(seq(',', alias($._quotedString, $.string))), '\''),

    _commaSeparatedString: $ => prec(1, seq($.string, repeat1(seq(',', $.string))))
  }

});
