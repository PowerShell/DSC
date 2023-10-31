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
    expression: $ => seq(field('function', $.function), field('members', optional($.memberAccess))),
    stringLiteral: $ => token(prec(PREC.STRINGLITERAL, /[^\[].*?/)),

    function: $ => seq(field('name', $.functionName), '(', field('args', optional($.arguments)), ')'),
    functionName: $ => /[a-z][a-zA-Z0-9]*/,
    arguments: $ => seq($._argument, repeat(seq(',', $._argument))),
    _argument: $ => choice($.expression, $._quotedString, $.number, $.boolean),

    _quotedString: $ => seq('\'', $.string, '\''),
    // ARM strings do not allow to contain single-quote characters
    string: $ => /[^']*/,
    number: $ => /-?\d+/,
    boolean: $ => choice('true', 'false'),

    memberAccess: $ => repeat1($._member),
    _member: $ => seq('.', $.memberName),
    memberName: $ => /[a-zA-Z0-9_-]+/,
  }

});
