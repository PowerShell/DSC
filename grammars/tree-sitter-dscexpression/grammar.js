const PREC = {
  ESCAPEDSTRING: 2,
  EXPRESSIONSTRING: 1,
  FUNCTION: 1,
  BOOLEAN: 0,
  STRINGLITERAL: -11,
}

module.exports = grammar({
  name: 'dscexpression',

  extras: $ => ['\n', ' '],

  rules: {
    statement: $ => choice(
      $.escapedStringLiteral,
      $._expressionString,
      $.stringLiteral,
    ),
    escapedStringLiteral: $ => token(prec(PREC.ESCAPEDSTRING, seq('[[', /.*?/))),
    _expressionString: $ => prec(PREC.EXPRESSIONSTRING, seq('[', $.expression, ']')),
    expression: $ => seq(field('function', $.function), optional(field('accessor',$.accessor))),
    stringLiteral: $ => token(prec(PREC.STRINGLITERAL, /[^\[](.|\n)*?/)),

    function: $ => prec(PREC.FUNCTION, seq(field('name', $.functionName), '(', field('args', optional($.arguments)), ')')),
    functionName: $ => choice(
      /[a-zA-Z][a-zA-Z0-9]*(\.[a-zA-Z0-9]+)?/,
      $._booleanLiteral
    ),
    arguments: $ => seq($._argument, repeat(seq(',', $._argument))),
    _argument: $ => choice($.expression, $._quotedString, $.number, $.boolean),

    _quotedString: $ => seq('\'', $.string, '\''),
    // ARM strings are not allowed to contain single-quote characters unless escaped
    string: $ => /([^']|''|\n)*/,
    number: $ => /-?\d+/,
    boolean: $ => prec(PREC.BOOLEAN, $._booleanLiteral),
    _booleanLiteral: $ => choice('true', 'false'),

    accessor: $ => repeat1(choice($.memberAccess, $.index)),

    memberAccess: $ => seq('.', field('name', $.memberName)),
    memberName: $ => /[a-zA-Z0-9_-]+/,

    propertyName: $ => seq('\'', field('string', $.string), '\''),
    index: $ => seq('[', field('indexValue', choice($.expression, $.number, $.propertyName)), ']'),
  }

});
