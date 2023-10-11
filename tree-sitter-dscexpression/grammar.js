const PREC = {
  ESCAPEDSTRING: 3,
  BRACKETINSTRING: 2,
  EXPRESSIONSTRING: 1,
  STRINGLITERAL: -11,
}

module.exports = grammar({
  name: 'dscexpression',

  rules: {
    statement: $ => choice(
      $.escapedStringLiteral,
      $.bracketInStringLiteral,
      $._expressionString,
      $.stringLiteral,
    ),
    escapedStringLiteral: $ => token(prec(PREC.ESCAPEDSTRING, seq('[[', /.*?/))),
    bracketInStringLiteral: $ => token(prec(PREC.BRACKETINSTRING, seq('[', /.*?/, ']', /.+?/))),
    _expressionString: $ => prec(PREC.EXPRESSIONSTRING, seq('[', $.expression, ']')),
    expression: $ => seq($.function, optional($._members)),
    stringLiteral: $ => token(prec(PREC.STRINGLITERAL, /[^\[].*?/)),

    function: $ => seq($.functionName, '(', optional($._arguments), ')'),
    functionName: $ => /[a-zA-Z]+/,
    _arguments: $ => seq($._argument, repeat(seq(',', $._argument))),
    _argument: $ => choice($.expression, $.string, $.number, $.boolean),

    string: $ => seq("'", /[^']*/, "'"),
    number: $ => /\d+/,
    boolean: $ => choice('true', 'false'),

    _members: $ => repeat1($._member),
    _member: $ => seq('.', $.memberName),
    memberName: $ => /[a-zA-Z0-9_-]+/,
  }

});
