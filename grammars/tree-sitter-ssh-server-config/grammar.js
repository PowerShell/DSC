//-------------------------------------------------------------------------------------------------------
// Copyright (C) Microsoft. All rights reserved.
// Licensed under the MIT license. See LICENSE file in the project root for full license information.
//-------------------------------------------------------------------------------------------------------

const PREC = {
  MATCH: 2,
  OPERATOR: 1
}

export default grammar({
  name: 'ssh_server_config',

  extras: $ => [' ', '\t', '\r', $.comment],

  rules: {
    server_config: $ => seq(repeat(choice($._new_line, $.keyword)), repeat($.match)),

    _new_line: $ => '\n',
    comment: $ => /#.*/,

    keyword: $ => seq(
      field('keyword', $.alphanumeric),
      choice(seq(/[ \t]/, optional('=')), '='),
      optional(field('operator', $.operator)),
      field('arguments', $.arguments),
      $._new_line
    ),

    match: $ => seq(
      token(prec(PREC.MATCH, /match/i)),
      seq(repeat1($.criteria), $._new_line),
      repeat1(choice($._new_line, $.keyword))
    ),

    criteria: $ => seq(
      field('keyword', $.alpha),
      choice(seq(/[ \t]/, optional('=')), '='),
      field('argument', alias($._argument, $.argument))
    ),

    _argument: $ => choice($.boolean, $.number, $.string, $._commaSeparatedString, $._doublequotedString, $._singlequotedString),
    arguments: $ => repeat1($._argument),

    alpha: $ => /[a-zA-Z]+/i,
    alphanumeric: $ => /[a-zA-Z0-9]+/i,
    boolean: $ => choice('yes', 'no'),
    number: $ => /\d+/,
    operator: $ => token(prec(PREC.OPERATOR, /[-+\^]/)),
    string: $ => /[^\n\r\s,"'#]+/, /* cannot contain spaces */

    _quotedString: $ => /[^\r\n,"']+/, /* can contain spaces */
    _doublequotedString: $ => seq('"', alias($._quotedString, $.string), repeat(seq(',', alias($._quotedString, $.string))), '"'),
    _singlequotedString: $ => seq('\'', alias($._quotedString, $.string), repeat(seq(',', alias($._quotedString, $.string))), '\''),

    _commaSeparatedString: $ => prec(1, seq($.string, repeat1(seq(',', $.string))))
  }

});
